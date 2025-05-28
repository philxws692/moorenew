use crate::system::sysinfo::get_hostname;
use crate::utils::configuration::{Configuration, LokiConfiguration};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::process;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_loki::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;

/// setup_run_logging sets the logging up, based on the set environment variables. If the variable
/// `STRUCTURED_LOGGING` is set, the logger will output the logs in JSON format, ready to
/// ingest into any SIEM or monitoring solution. If the variable is not set, it will pretty print
/// the log messages.
/// This logging configuration is used for when the program is run as a service or the run command
/// is used.
/// # Examples
/// ```rs
/// use crate::utils::logging;
///
/// fn main() {
///     logging::setup_run_logging();
/// }
/// ```
pub fn setup_run_logging(level: &str, configuration: &Configuration) {
    let logging_level = match &*level.to_lowercase() {
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    // Basis-Registry
    let subscriber = tracing_subscriber::registry().with(logging_level);

    // Check if loki logging is configured
    let loki_logging = configuration.logging.loki.is_some();

    // Structured logging
    if configuration.logging.structured_logging & loki_logging {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .with(get_loki_layer(configuration.logging.loki.as_ref().unwrap()))
            .init();
        debug!("Structured and loki logging enabled");
    } else if !configuration.logging.structured_logging & loki_logging {
        subscriber
            .with(tracing_subscriber::fmt::layer())
            .with(get_loki_layer(configuration.logging.loki.as_ref().unwrap()))
            .init();
        debug!("Structured logging disabled and loki logging enabled");
    } else if configuration.logging.structured_logging & !loki_logging {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init();
        debug!("Structured logging enabled and loki logging disabled");
    } else {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
        debug!("Structured logging and loki logging disabled");
    }
}

fn get_loki_layer(loki_configuration: &LokiConfiguration) -> Layer {
    let url = Url::parse(&loki_configuration.url).expect("Failed to parse Grafana URL");

    let user = &loki_configuration.user;
    let pass = &loki_configuration.password;

    let (loki_layer, task);

    if !user.is_empty() && !pass.is_empty() {
        let basic_auth = format!("{user}:{pass}");
        let encoded_basic_auth = BASE64_STANDARD.encode(basic_auth.as_bytes());

        (loki_layer, task) = tracing_loki::builder()
            .label("application", "moorenew")
            .expect("Failed labeling the layer")
            .extra_field("pid", format!("{}", process::id()))
            .expect("Failed adding pid field")
            .extra_field("host", get_hostname())
            .expect("Failed adding hostname field")
            .http_header("Authorization", format!("Basic {}", encoded_basic_auth))
            .expect("Failed to add Authorization header to the request")
            .build_url(url)
            .expect("Failed to build Grafana URL");
    } else {
        (loki_layer, task) = tracing_loki::builder()
            .label("application", "moorenew")
            .expect("Failed labeling the layer")
            .extra_field("pid", format!("{}", process::id()))
            .expect("Failed adding pid field")
            .build_url(url)
            .expect("Failed to build Grafana URL");
    }

    tokio::spawn(task);

    loki_layer
}

/// setup_basic_logging sets the logging up, based on the set environment variables. This logging
/// configuration is used for when the user interacts with the program
pub fn setup_basic_logging(logging_level: LevelFilter) {
    tracing_subscriber::registry()
        .with(logging_level)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
