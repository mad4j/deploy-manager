use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use tracing::info;

use crate::core::config::{ArchiveActionConfig, ArchiveFormat, ArchiveOperation};
use crate::core::executor::ExecutionContext;

/// Execute an `archive` action.
pub async fn run(cfg: &ArchiveActionConfig, ctx: &ExecutionContext) -> Result<()> {
    info!(
        operation = ?cfg.operation,
        format = ?cfg.format,
        source = %cfg.source.display(),
        destination = %cfg.destination.display(),
        "Archive action"
    );

    if ctx.dry_run {
        info!(
            "[dry-run] Would {} {:?} archive: {} -> {}",
            match cfg.operation {
                ArchiveOperation::Create => "create",
                ArchiveOperation::Extract => "extract",
            },
            cfg.format,
            cfg.source.display(),
            cfg.destination.display()
        );
        return Ok(());
    }

    match cfg.operation {
        ArchiveOperation::Create => create_archive(cfg),
        ArchiveOperation::Extract => extract_archive(cfg),
    }
}

// ---------------------------------------------------------------------------
// Create helpers
// ---------------------------------------------------------------------------

fn create_archive(cfg: &ArchiveActionConfig) -> Result<()> {
    if cfg.destination.exists() && !cfg.overwrite {
        anyhow::bail!(
            "Archive destination already exists (set overwrite: true to replace): {}",
            cfg.destination.display()
        );
    }
    if let Some(parent) = cfg.destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for {}", cfg.destination.display()))?;
    }

    match cfg.format {
        ArchiveFormat::Zip => create_zip(&cfg.source, &cfg.destination),
        ArchiveFormat::TarGz => create_tar_gz(&cfg.source, &cfg.destination),
    }
}

fn create_zip(source: &Path, destination: &Path) -> Result<()> {
    let file = File::create(destination)
        .with_context(|| format!("Failed to create archive file: {}", destination.display()))?;
    let writer = BufWriter::new(file);
    let mut zip = zip::ZipWriter::new(writer);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    if source.is_dir() {
        add_dir_to_zip(&mut zip, source, source, options)?;
    } else {
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        zip.start_file(file_name, options)
            .context("Failed to start zip file entry")?;
        let mut f = File::open(source)
            .with_context(|| format!("Failed to open source file: {}", source.display()))?;
        io::copy(&mut f, &mut zip).context("Failed to write file into archive")?;
    }

    zip.finish().context("Failed to finalize zip archive")?;
    info!(path = %destination.display(), "Zip archive created");
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<BufWriter<File>>,
    base: &Path,
    dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let relative = path.strip_prefix(base).context("Failed to compute relative path")?;
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            zip.add_directory(&relative_str, options)
                .context("Failed to add directory to zip")?;
            add_dir_to_zip(zip, base, &path, options)?;
        } else {
            zip.start_file(&relative_str, options)
                .context("Failed to start zip file entry")?;
            let mut f = File::open(&path)
                .with_context(|| format!("Failed to open file: {}", path.display()))?;
            io::copy(&mut f, zip).context("Failed to write file into archive")?;
        }
    }
    Ok(())
}

fn create_tar_gz(source: &Path, destination: &Path) -> Result<()> {
    let file = File::create(destination)
        .with_context(|| format!("Failed to create archive file: {}", destination.display()))?;
    let gz_encoder = flate2::write::GzEncoder::new(BufWriter::new(file), flate2::Compression::default());
    let mut archive = tar::Builder::new(gz_encoder);

    if source.is_dir() {
        let dir_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("dir");
        archive
            .append_dir_all(dir_name, source)
            .with_context(|| format!("Failed to add directory to tar: {}", source.display()))?;
    } else {
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let mut f = File::open(source)
            .with_context(|| format!("Failed to open source file: {}", source.display()))?;
        archive
            .append_file(file_name, &mut f)
            .with_context(|| format!("Failed to add file to tar: {}", source.display()))?;
    }

    archive.finish().context("Failed to finalize tar.gz archive")?;
    info!(path = %destination.display(), "Tar.gz archive created");
    Ok(())
}

// ---------------------------------------------------------------------------
// Extract helpers
// ---------------------------------------------------------------------------

fn extract_archive(cfg: &ArchiveActionConfig) -> Result<()> {
    if cfg.destination.exists() && !cfg.overwrite {
        anyhow::bail!(
            "Extract destination already exists (set overwrite: true to replace): {}",
            cfg.destination.display()
        );
    }
    fs::create_dir_all(&cfg.destination)
        .with_context(|| format!("Failed to create destination directory: {}", cfg.destination.display()))?;

    match cfg.format {
        ArchiveFormat::Zip => extract_zip(&cfg.source, &cfg.destination),
        ArchiveFormat::TarGz => extract_tar_gz(&cfg.source, &cfg.destination),
    }
}

fn extract_zip(source: &Path, destination: &Path) -> Result<()> {
    let file = File::open(source)
        .with_context(|| format!("Failed to open archive: {}", source.display()))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i).context("Failed to access zip entry")?;
        let out_path = destination.join(zip_file.mangled_name());

        if zip_file.is_dir() {
            fs::create_dir_all(&out_path)
                .with_context(|| format!("Failed to create directory: {}", out_path.display()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&out_path)
                .with_context(|| format!("Failed to create file: {}", out_path.display()))?;
            io::copy(&mut zip_file, &mut out_file).context("Failed to extract zip entry")?;
        }
    }

    info!(path = %destination.display(), "Zip archive extracted");
    Ok(())
}

fn extract_tar_gz(source: &Path, destination: &Path) -> Result<()> {
    let file = File::open(source)
        .with_context(|| format!("Failed to open archive: {}", source.display()))?;
    let gz_decoder = flate2::read::GzDecoder::new(BufReader::new(file));
    let mut archive = tar::Archive::new(gz_decoder);
    archive
        .unpack(destination)
        .with_context(|| format!("Failed to extract tar.gz into: {}", destination.display()))?;

    info!(path = %destination.display(), "Tar.gz archive extracted");
    Ok(())
}
