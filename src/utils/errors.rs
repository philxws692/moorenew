use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum MoorenewError {
    #[error("failed to create systemd timer file")]
    TimerFileCreationFailed(#[source] std::io::Error),
    
    #[error("failed to create systemd service file")]
    ServiceFileCreationFailed(#[source] std::io::Error),
    
    #[error("see components field for more information")]
    ServiceConfigGenerationFailed{
        components: Vec<String>
    },
    
}