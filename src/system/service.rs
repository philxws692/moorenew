use crate::system::serviceproviders::ServiceProvider;
use crate::system::sysinfo::get_binary_path;
use crate::utils::errors::MoorenewError;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::string::String;
use tracing::{debug, error};

pub fn create_service_files(
    service_name: &str,
    service_provider: ServiceProvider,
    force: bool,
) -> Result<(), MoorenewError> {
    // TODO: enable passing the seconds as arguments
    let mut errored = false;
    let mut errored_creations: Vec<String> = Vec::new();
    match service_provider {
        ServiceProvider::SYSTEMD => {
            match create_systemd_timer_file(service_name, 60, 60 * 60, force) {
                Ok(_) => {}
                Err(_) => {
                    errored = true;
                    errored_creations.push("timer".to_string());
                }
            }
            match create_systemd_service_file(service_name, force) {
                Ok(_) => {}
                Err(_) => {
                    errored = true;
                    errored_creations.push("service".to_string());
                }
            }
        }
        ServiceProvider::RC => {
            // TODO: Implement RC file configurations
        }
    }

    if errored {
        return Err(MoorenewError::ServiceConfigGenerationFailed {
            components: errored_creations,
        });
    }

    Ok(())
}

fn create_systemd_timer_file(
    service_name: &str,
    on_boot_sec: isize,
    on_unit_active_sec: isize,
    force: bool,
) -> Result<(), MoorenewError> {
    let timer_file_string = format!(
        "[Unit]
Description=\"Copying SSL certificates for mailcow\"

[Timer]
OnUnitActiveSec={on_unit_active_sec}s
OnBootSec={on_boot_sec}s
Unit={service_name}.service

[Install]
WantedBy=multi-user.target\n"
    );

    let timer_file_name = format!("{service_name}.timer");

    if Path::new(&timer_file_name).exists() && !force {
        let msg = "timer file already exists. run with -f flag to overwrite";
        error!(file = %timer_file_name, error = msg, "Could not create timer file");
        return Err(MoorenewError::TimerFileCreationFailed(Error::new(
            std::io::ErrorKind::AlreadyExists,
            msg,
        )));
    }

    let mut file = File::create(&timer_file_name).map_err(|e| {
        error!(error = %e, file = %timer_file_name, "Could not create timer file");
        MoorenewError::TimerFileCreationFailed(e)
    })?;

    file.write_all(timer_file_string.as_bytes()).map_err(|e| {
        error!(error = %e, file = %timer_file_name, "Could not write to timer file");
        MoorenewError::TimerFileCreationFailed(e)
    })?;

    debug!(file = %timer_file_name, "Timer file created successfully");
    Ok(())
}

fn create_systemd_service_file(service_name: &str, force: bool) -> Result<(), MoorenewError> {
    let binary_path: String;

    let service_file_name = format!("{service_name}.service");

    if Path::new(&service_file_name).exists() && !force {
        let msg = "service file already exists. run with -f flag to overwrite";
        error!(file = %service_file_name, error = msg, "Could not create service file");
        return Err(MoorenewError::ServiceFileCreationFailed(Error::new(
            std::io::ErrorKind::AlreadyExists,
            msg,
        )));
    }

    match get_binary_path() {
        Ok(path) => {
            binary_path = path;
        }
        Err(_) => {
            error!(
                "Could not get binary path, please set it manually in the {service_name}.service file"
            );
            binary_path = "<set binary path here>".to_string()
        }
    }

    let service_file_string = format!(
        "[Unit]
Description=updates the ssl certificates for mailcow
After=network.target
StartLimitIntervalSec=0
[Service]
Type=simple
Restart=on-failure
RestartSec=1
User=root
ExecStart={binary_path}

[Install]
WantedBy=multi-user.target\n"
    );

    let mut file = File::create(&service_file_name).map_err(|e| {
        error!(error = %e, file = %service_file_name, "Could not create service file");
        MoorenewError::ServiceFileCreationFailed(e)
    })?;

    file.write_all(service_file_string.as_bytes())
        .map_err(|e| {
            error!(error = %e, file = %service_file_name, "Could not write to service file");
            MoorenewError::ServiceFileCreationFailed(e)
        })?;

    debug!(file = %service_file_name, "Service file created successfully");
    Ok(())
}
