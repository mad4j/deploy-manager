use indicatif::MultiProgress;
use std::io::{self, Write};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, MakeWriter},
    prelude::*,
};

#[derive(Clone)]
struct ProgressMakeWriter {
    progress: Option<MultiProgress>,
}

impl<'a> MakeWriter<'a> for ProgressMakeWriter {
    type Writer = ProgressLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        ProgressLogWriter {
            progress: self.progress.clone(),
            buffer: Vec::new(),
        }
    }
}

struct ProgressLogWriter {
    progress: Option<MultiProgress>,
    buffer: Vec<u8>,
}

impl ProgressLogWriter {
    fn emit_line(&self, line: &[u8]) -> io::Result<()> {
        let text = String::from_utf8_lossy(line);
        let message = text.trim_end_matches(['\r', '\n']);

        if let Some(progress) = &self.progress {
            progress.suspend(|| {
                let mut stderr = io::stderr().lock();
                writeln!(stderr, "{message}")
            })
        } else {
            let mut stderr = io::stderr().lock();
            writeln!(stderr, "{message}")
        }
    }
}

impl Write for ProgressLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        while let Some(pos) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line = self.buffer.drain(..=pos).collect::<Vec<_>>();
            self.emit_line(&line)?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            let line = std::mem::take(&mut self.buffer);
            self.emit_line(&line)?;
        }

        io::stderr().lock().flush()
    }
}

/// Initialise structured logging.
///
/// Log level is driven by the `RUST_LOG` environment variable
/// (default: `info`).  When `verbose` is true the filter is forced to
/// `debug`.
pub fn init(verbose: bool, progress: Option<MultiProgress>) {
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(false)
                .compact()
                .with_writer(ProgressMakeWriter { progress }),
        )
        .init();
}
