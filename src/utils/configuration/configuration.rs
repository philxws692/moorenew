use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub sftp_host: String,
    pub sftp_port: u16,
    pub sftp_user: String,
    pub private_key_path: String,
    pub public_key_path: String,
    pub npm_cert_path: String,
    pub mail_cert_path: String,
    pub logging: LoggingConfiguration,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoggingConfiguration {
    #[serde(default = "default_logging_level")]
    pub level: String,
    #[serde(default = "default_structured_logging")]
    pub structured_logging: bool,
    pub loki: Option<LokiConfiguration>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LokiConfiguration {
    pub url: String,
    pub user: String,
    pub password: String,
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration {
            sftp_host: String::from("localhost"),
            sftp_port: 22,
            sftp_user: String::from("user"),
            private_key_path: String::from("private_key.pem"),
            public_key_path: String::from("public_key.pem"),
            npm_cert_path: String::from("npm_cert.pem"),
            mail_cert_path: String::from("mail_cert.pem"),
            logging: LoggingConfiguration {
                level: String::from("info"),
                structured_logging: false,
                loki: None,
            },
        }
    }

    pub fn write_to_file(&self) {
        match toml::to_string(&self) {
            Ok(toml_string) => {
                let user_path = std::env::home_dir().unwrap();
                let config_path = user_path.join(".moorenew/config.toml");
                std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
                match std::fs::File::create(config_path) {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(toml_string.as_bytes()) {
                            eprintln!("Error writing to config file: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Error creating config file: {}", e),
                }
            }
            Err(e) => eprintln!("Error parsing configuration: {}", e),
        }
    }
}

pub fn read_config_from_file() -> Option<Configuration> {
    let user_path = std::env::home_dir().unwrap();
    let config_path = user_path.join(".moorenew/config.toml");

    match toml::from_str::<Configuration>(&std::fs::read_to_string(config_path).unwrap()) {
        Ok(config) => Some(config),
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            None
        }
    }
}

/*
 * Default values
 */
fn default_logging_level() -> String {
    "info".to_string()
}

fn default_structured_logging() -> bool {
    false
}
