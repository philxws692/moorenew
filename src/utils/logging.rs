use std::{env, process};
use base64::Engine;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_loki::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;
use base64::prelude::BASE64_STANDARD;
use crate::system::sysinfo::get_hostname;

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
pub fn setup_run_logging() {
    let logging_level;

    match env::var("LOGGING_LEVEL") {
        Ok(level) => match &*level.to_lowercase() {
            "info" => logging_level = LevelFilter::INFO,
            "debug" => logging_level = LevelFilter::DEBUG,
            "trace" => logging_level = LevelFilter::TRACE,
            "warn" => logging_level = LevelFilter::WARN,
            "error" => logging_level = LevelFilter::ERROR,
            _ => logging_level = LevelFilter::INFO,
        },
        Err(_) => {
            logging_level = LevelFilter::INFO;
        }
    }

    // Basis-Registry
    let subscriber = tracing_subscriber::registry().with(logging_level);

    // Structured logging
    if (env::var("STRUCTURED_LOGGING").unwrap_or_else(|_| "false".into()) == "true")
        && (env::var("LOKI_LOGGING").unwrap_or_else(|_| "false".into()) == "true")
    {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .with(get_loki_layer())
            .init();
        debug!("Structured and loki logging enabled");
    } else if (env::var("STRUCTURED_LOGGING").unwrap_or_else(|_| "false".into()) == "false")
        && (env::var("LOKI_LOGGING").unwrap_or_else(|_| "false".into()) == "true")
    {
        subscriber
            .with(tracing_subscriber::fmt::layer())
            .with(get_loki_layer())
            .init();
        debug!("Structured logging disabled and loki logging enabled");
    } else if (env::var("STRUCTURED_LOGGING").unwrap_or_else(|_| "false".into()) == "true")
        && (env::var("LOKI_LOGGING").unwrap_or_else(|_| "false".into()) == "false")
    {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init();
        debug!("Structured logging enabled and loki logging disabled");
    } else {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
        debug!("Structured logging and loki logging disabled");
    }
}

fn get_loki_layer() -> Layer {
    let env_url = env::var("LOKI_LOGGING_URL").unwrap_or_else(|_| "http://127.0.0.1".into());
    let url= Url::parse(format!("{}:3100", env_url).as_str()).expect("Failed to parse Grafana URL");
    if env_url == "http://127.0.0.1" {
        println!("environment variable LOKI_LOGGING_URL is set to default value, please set it to your Loki URL")
    }

    let user = env::var("LOKI_USER").unwrap_or_else(|_| "".into());
    let pass = env::var("LOKI_PASSWORD").unwrap_or_else(|_| "".into());

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
            .expect(
                "Failed to add Authorization header to the request",
            )
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
    tracing_subscriber::registry().with(logging_level).with(tracing_subscriber::fmt::layer()).init();
}