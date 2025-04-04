use std::env;
use std::fs::File;
use std::io::{Read, Write};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;
use tracing::{error, info, warn};

pub struct SftpClient {
    session: Session,
    socket: TcpStream,
}

impl SftpClient {
    pub fn connect(username: &str, host: &str, private_key: &str, public_key: &str) -> std::io::Result<Self> {
        let private_key_path = Path::new(private_key);
        let public_key_path = Path::new(public_key);

        // Connect to SFTP
        let tcp = TcpStream::connect(format!("{}:{}", host, env::var("SFTP_PORT").unwrap()))?;
        let mut session = Session::new()?;

        session.set_tcp_stream(tcp.try_clone()?);
        session.handshake()?;

        // Login with publickey
        session
            .userauth_pubkey_file(username, Some(public_key_path), private_key_path, None)?;

        if !session.authenticated() {
            error!("authentication failed");
            return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "ssh_authentication failed"));
        }

        info!("connected to sftp server at {}", host);

        Ok(Self { session, socket: tcp })
    }

    pub fn disconnect(self){
        drop(self.session);
        if let Err(e) = self.socket.shutdown(std::net::Shutdown::Both) {
            warn!("ssh_disconnect error: {}", e);
        }
    }


    pub fn download_file(&self, remote_path: &str, local_path: &str) -> std::io::Result<()> {
        info!("downloading {} to {} from {}", remote_path, local_path, self.socket.peer_addr()?);
        let sftp = self.session.sftp()?;

        let mut remote_file;
        let mut local_file;

        match sftp.open(remote_path) {
            Ok(file) => {remote_file = file},
            Err(e) => {
                warn!("sftp open error: {}", e);
                exit(1)
            }
        }

        match File::create(local_path) {
            Ok(file) => {local_file = file},
            Err(e) => {
                warn!("can not create local file: {}", e);
                exit(1)
            }
        }

        let mut buffer = Vec::new();
        remote_file.read_to_end(&mut buffer)?;
        local_file.write_all(&buffer)?;

        Ok(())
    }

}
