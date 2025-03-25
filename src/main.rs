mod utils;

use crate::utils::sftp::SftpClient;
use dotenv::dotenv;
use std::env;
use std::env::VarError;
use tracing::{error, info};
use tracing_subscriber::fmt::Subscriber;

fn main() {
    dotenv().ok();

    setup_logging();

    let username = &env::var("SFTP_USERNAME").unwrap()[..];
    let host = &env::var("SFTP_HOST").unwrap()[..];
    let private_key_path = &env::var("PRIVATE_KEY_PATH").unwrap()[..];
    let public_key_path = &env::var("PUBLIC_KEY_PATH").unwrap()[..];

    let client = SftpClient::connect(username, host, private_key_path, public_key_path).unwrap();


}

fn setup_logging() {
    let structured_logging_key = "STRUCTURED_LOGGING";
    match env::var(structured_logging_key) {
        Ok(value) => {
            if value == "true" {
                // Structured logging enabled

                let subscriber = tracing_subscriber::fmt().json().finish();
                tracing::subscriber::set_global_default(subscriber).unwrap();

                info!("logging is set to {}", structured_logging_key);
            } else {
                tracing_subscriber::fmt().init();
                info!("structured logging disabled");
            }
        }
        Err(_) => {
            unsafe {
                env::set_var(structured_logging_key, "false");
            }
            assert_eq!(env::var(structured_logging_key), Ok("false".to_string()));
            tracing_subscriber::fmt().init();
            info!("structured logging disabled");
        }
    }
}
