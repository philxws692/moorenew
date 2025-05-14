use crate::system::serviceproviders::ServiceProvider;
use crate::utils::sysinfo::get_binary_path;
use std::fs::File;
use std::io::{Error, Write};
use tracing::{debug, error};

pub fn create_service_files(
    service_name: &str,
    service_provider: ServiceProvider,
) -> Result<(), Error> {
    // TODO: enable passing the seconds as arguments
    match service_provider {
        ServiceProvider::SYSTEMD => {
            create_systemd_timer_file(service_name, 60, 60 * 60);
            create_systemd_service_file(service_name);
        }
        ServiceProvider::RC => {
            // TODO: Implement RC file configurations
        }
    }

    Ok(())
}

fn create_systemd_timer_file(service_name: &str, on_boot_sec: isize, on_unit_active_sec: isize) {
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

    match File::create(format!("{service_name}.timer")) {
        Ok(mut file) => match file.write_all(timer_file_string.as_bytes()) {
            Ok(_) => {
                debug!("Timer file created")
            }
            Err(_) => {
                error!("Could not write timer file")
            }
        },
        Err(_) => {
            error!("Could not create timer file")
        }
    }
}

fn create_systemd_service_file(service_name: &str) {
    let binary_path: String;

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

    match File::create(format!("{service_name}.service")) {
        Ok(mut file) => match file.write_all(service_file_string.as_bytes()) {
            Ok(_) => {
                debug!("Service file created")
            }
            Err(_) => {
                error!("Could not write service file")
            }
        },
        Err(_) => {
            error!("Could not create service file")
        }
    }
}
