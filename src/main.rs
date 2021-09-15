use edxlc::config::Config;
use log::{debug, info};
use std::env;

const CONFIG_FILENAME: &str = "edxlc.toml";

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: &str = "edxlc=debug";
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: &str = "info";

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    configure_logger();
    info!("EDXLC {}", VERSION);

    edxlc::config::write_default_file_if_missing(CONFIG_FILENAME);
    let config = Config::from_file(config_filename());
    debug!("{:?}", config);

    edxlc::run(config);
}

fn config_filename() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let config_filename = String::from(CONFIG_FILENAME);
        debug!("Using default configuration filename: {}", config_filename);
        config_filename
    } else {
        let config_filename = &args[1];
        debug!(
            "Using command line configuration filename: {}",
            config_filename
        );
        config_filename.clone()
    }
}

fn configure_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_LEVEL))
        .init();
}
