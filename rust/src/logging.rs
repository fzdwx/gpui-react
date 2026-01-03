use logforth::append;
use logforth::filter;
use logforth::layout::TextLayout;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize logging system with logforth.
/// This function is idempotent - it will only initialize once.
/// Configuration:
/// - ERROR and WARN logs go to stderr
/// - INFO, DEBUG, and TRACE logs go to stdout
/// - Log level is controlled via RUST_LOG environment variable
/// - Timestamps are displayed in local timezone (e.g., 2024-08-11 22:44:57.172 +08:00)
pub fn init_logging() {
    INIT.call_once(|| {
        let env_filter_stdout = filter::EnvFilter::from_default_env();
        let env_filter_stderr = filter::EnvFilter::from_default_env();
        let layout = TextLayout::default();
        logforth::builder()
            .dispatch(|d| {
                d.filter(env_filter_stderr)
                    .append(append::Stderr::default().with_layout(layout.clone()))
            })
            .dispatch(|d| {
                d.filter(env_filter_stdout)
                    .append(append::Stdout::default().with_layout(layout))
            })
            .apply();

        log::info!("Logging system initialized");
    });
}
