use sha2::{Digest, Sha256};
use std::fs::File;
use std::io;
use tracing::error;

pub trait FileExt {
    fn sha256(self) -> io::Result<String>;
}

impl FileExt for File {
    fn sha256(mut self) -> io::Result<String> {
        let mut hasher = Sha256::new();
        match io::copy(&mut self, &mut hasher) {
            Ok(_) => {
                let hash = hasher.finalize();
                Ok(hash.iter().map(|b| format!("{:02x}", b)).collect())
            }
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::fileext::FileExt;
    use std::fs::File;

    #[test]
    fn test_sha256() {
        assert_eq!(
            File::open("tests/file1.txt").unwrap().sha256().unwrap(),
            "3cc2306e83e37c18cc33adf4b2672470aaf8c9ecd3b55ad78927544fff5215e5"
        );
        assert_eq!(
            File::open("tests/file2.txt").unwrap().sha256().unwrap(),
            "3cc2306e83e37c18cc33adf4b2672470aaf8c9ecd3b55ad78927544fff5215e5"
        );
        assert_eq!(
            File::open("tests/file3.txt").unwrap().sha256().unwrap(),
            "2f7fd0cf462dacfce91c0459af54ad01c5266324d6b13087d3380636cb47f7a7"
        );
        assert_ne!(
            File::open("tests/file3.txt").unwrap().sha256().unwrap(),
            "3cc2306e83e37c18cc33adf4b2672470aaf8c9ecd3b55ad78927544fff5215e5"
        );
    }
}
