use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum MoorenewError {
    #[error("failed to create systemd timer file")]
    TimerFileCreationFailed(#[source] std::io::Error),

    #[error("failed to create systemd service file")]
    ServiceFileCreationFailed(#[source] std::io::Error),

    #[error("see components field for more information")]
    ServiceConfigGenerationFailed { components: Vec<String> },

    #[error("could not execute local command `{command}`: {error}")]
    LocalCommandExecutionError {
        command: String,
        error: std::io::Error,
    },

    #[error("could not connect to ssh host")]
    SSHConnectError(#[source] std::io::Error),

    #[error("error executing remote command")]
    SSHExecutionError(#[source] ssh2::Error),

    #[error("configuration error")]
    ConfigurationError(#[source] ConfigurationError),

    #[error("error getting sha256 checksum")]
    CalculatingChecksum(#[source] std::io::Error),

    #[error("error transferring file")]
    FileTransfer(#[source] std::io::Error),

    #[error("invalid utf8 output from `{command}`")]
    Utf8Output {
        command: String,
        error: std::string::FromUtf8Error,
    },

    #[error("loki logging configuration error")]
    LokiConfigurationError(#[source] anyhow::Error),

    #[error("an unknown error occurred: {0}")]
    Unknown(#[source] anyhow::Error),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("enabled loki logging, but loki configuration is missing")]
    LokiEnabledConfigurationMissing,

    #[error("unable to create configuration directory")]
    DirectoryCreation(#[source] std::io::Error),

    #[error("error with config file creation")]
    ConfigFileCreation(#[source] std::io::Error),

    #[error("could not serialize config")]
    ConfigSerialization(#[source] toml::ser::Error),

    #[error("could not parse config")]
    ConfigParsing(#[source] toml::de::Error),

    #[error("could not determine home directory")]
    HomeDirUnavailable,
}
