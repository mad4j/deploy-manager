use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialise structured logging.
///
/// Log level is driven by the `RUST_LOG` environment variable
/// (default: `info`).  When `verbose` is true the filter is forced to
/// `debug`.
pub fn init(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).compact())
        .init();
}
