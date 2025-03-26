use std::fs;
use sha2::{Digest, Sha256};

pub fn files_identical(curr_cert: &str, new_cert: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(&fs::read(curr_cert).unwrap());

    let curr_cert_sha = hasher.finalize();

    let mut hasher2 = Sha256::new();
    hasher2.update(&fs::read(new_cert).unwrap());
    let new_cert_sha = hasher2.finalize();

    if curr_cert_sha != new_cert_sha {
        return false
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::utils::hashes::files_identical;

    #[test]
    fn test_files_identical() {
        assert_eq!(true, files_identical("tests/file1.txt", "tests/file1.txt"));
        assert_eq!(true, files_identical("tests/file2.txt", "tests/file1.txt"));
        assert_eq!(false, files_identical("tests/file3.txt", "tests/file1.txt"));
    }
}