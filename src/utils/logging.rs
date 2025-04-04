use std::env;
use tracing::{debug, info, Level};
use tracing::level_filters::LevelFilter;

/// setup_logging sets the logging up, based on the set environment variables. If the variable
/// `STRUCTURED_LOGGING` is set, the logger will output the logs in JSON format, ready to
/// ingest into any SIEM or monitoring solution. If the variable is not set it will pretty print
/// the log messages.
/// # Examples
/// ```rs
/// use crate::utils::logging;
///
/// fn main() {
///     logging::setup_logging();
/// }
/// ```
pub fn setup_logging() {
    let structured_logging_key = "STRUCTURED_LOGGING";

    let logging_level;

    match env::var("LOGGING_LEVEL") {
        Ok(level) => {
            match &*level.to_lowercase() {
                "info" => logging_level = LevelFilter::INFO,
                "debug" => logging_level = LevelFilter::DEBUG,
                "trace" => logging_level = LevelFilter::TRACE,
                "warn" => logging_level = LevelFilter::WARN,
                "error" => logging_level = LevelFilter::ERROR,
                _ => logging_level = LevelFilter::INFO,
            }
        }
        Err(_) => {
            logging_level = LevelFilter::INFO;
        }
    }

    match env::var(structured_logging_key) {
        Ok(value) => {
            if value == "true" {
                // Structured logging enabled

                let subscriber = tracing_subscriber::fmt().json().with_max_level(logging_level).finish();
                tracing::subscriber::set_global_default(subscriber).unwrap();

                debug!("logging is set to {}", structured_logging_key);
            } else {
                tracing_subscriber::fmt().with_max_level(logging_level).init();
                debug!("structured logging disabled");
            }
        }
        Err(_) => {
            unsafe {
                env::set_var(structured_logging_key, "false");
            }
            assert_eq!(env::var(structured_logging_key), Ok("false".to_string()));
            tracing_subscriber::fmt().with_max_level(logging_level).init();
            debug!("structured logging disabled");
        }
    }
}