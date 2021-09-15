use edxlc::config::Config;
use log::{debug, info};

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: &str = "edxlc=debug";
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: &str = "info";

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    configure_logger();
    info!("EDXLC {}", VERSION);

    edxlc::config::write_default_file_if_missing();
    let config = Config::from_file();
    debug!("{:?}", config);

    edxlc::run(config);
}

fn configure_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_LEVEL))
        .init();
}
