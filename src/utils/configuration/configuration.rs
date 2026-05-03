use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};

use crate::utils::errors::{self, ConfigurationError, MoorenewError};

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
    #[serde(default = "default_containers")]
    pub containers: Vec<String>,
    pub buzz_urls: Vec<String>,
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
            buzz_urls: vec![
                String::from("gotify://myawesome.gotify.com/myawesomepath/myawesometoken"),
                String::from("ntfy://username:password@ntfy.host/mytopic"),
            ],
            containers: default_containers(),
        }
    }

    pub fn write_to_file(&self) -> Result<(), MoorenewError> {
        let config_string = toml::to_string(&self).map_err(|e| {
            MoorenewError::ConfigurationError(errors::ConfigurationError::ConfigSerialization(e))
        })?;
        let user_path = std::env::home_dir().ok_or_else(|| {
            MoorenewError::ConfigurationError(ConfigurationError::HomeDirUnavailable)
        })?;
        let config_path = user_path.join(".moorenew/config.toml");
        std::fs::create_dir_all(user_path.join(".moorenew")).map_err(|e| {
            MoorenewError::ConfigurationError(ConfigurationError::DirectoryCreation(e))
        })?;

        let mut config_file = File::create(config_path).map_err(|e| {
            MoorenewError::ConfigurationError(ConfigurationError::ConfigFileCreation(e))
        })?;

        config_file
            .write_all(config_string.as_bytes())
            .map_err(|e| {
                MoorenewError::ConfigurationError(errors::ConfigurationError::ConfigFileCreation(e))
            })
    }
}

pub fn read_config_from_file() -> Result<Configuration, MoorenewError> {
    let user_path = std::env::home_dir()
        .ok_or_else(|| MoorenewError::ConfigurationError(ConfigurationError::HomeDirUnavailable))?;
    let config_path = user_path.join(".moorenew/config.toml");

    let config_contents = std::fs::read_to_string(config_path).map_err(|e| {
        MoorenewError::ConfigurationError(errors::ConfigurationError::ConfigFileCreation(e))
    })?;

    toml::from_str::<Configuration>(&config_contents).map_err(|e| {
        MoorenewError::ConfigurationError(errors::ConfigurationError::ConfigParsing(e))
    })
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

fn default_containers() -> Vec<String> {
    vec![
        String::from("postfix-mailcow"),
        String::from("dovecot-mailcow"),
        String::from("nginx-mailcow"),
    ]
}
