use logforth::append;
use logforth::filter;
use logforth::layout::TextLayout;
use std::sync::Once;

static INIT: Once = Once::new();

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
