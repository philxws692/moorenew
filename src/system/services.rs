use crate::system::serviceproviders::ServiceProvider;
use crate::system::sysinfo::get_binary_path;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use tracing::{debug, error};

pub fn create_service_files(
    service_name: &str,
    service_provider: ServiceProvider,
    force: bool
) -> Result<(), Error> {
    // TODO: enable passing the seconds as arguments
    match service_provider {
        ServiceProvider::SYSTEMD => {
            create_systemd_timer_file(service_name, 60, 60 * 60, force);
            create_systemd_service_file(service_name, force);
        }
        ServiceProvider::RC => {
            // TODO: Implement RC file configurations
        }
    }

    Ok(())
}

fn create_systemd_timer_file(service_name: &str, on_boot_sec: isize, on_unit_active_sec: isize, force: bool) {
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
        error!(error = "Timer file already exists. Run with -f flag to overwrite", "Could not create timer file");
        return
    }

    match File::create(timer_file_name.clone()) {
        Ok(mut file) => match file.write_all(timer_file_string.as_bytes()) {
            Ok(_) => {
                debug!(filename = timer_file_name, "Timer file created")
            }
            Err(e) => {
                error!(error = e.to_string(), "Could not write to timer file");
                return
            }
        },
        Err(e) => {
            error!(error = e.to_string(), "Could not create timer file");
        }
    }
}

fn create_systemd_service_file(service_name: &str, force: bool) {
    let binary_path: String;
    
    let service_file_name = format!("{service_name}.service");
    
    if Path::new(&service_file_name).exists() && !force {
        error!(error = "Service file already exists. run with -f flag to overwrite", "Could not create service file");
        return  
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

    match File::create(format!("{service_name}.service")) {
        Ok(mut file) => match file.write_all(service_file_string.as_bytes()) {
            Ok(_) => {
                debug!("Service file created")
            }
            Err(e) => {
                error!(error = e.to_string(), "Could not write to service file");
                return
            }
        },
        Err(e) => {
            error!(error = e.to_string(), "Could not create service file");
        }
    }
}
