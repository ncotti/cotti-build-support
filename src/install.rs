use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use tar::Archive;
use std::fs;

/// Copies all files from "src_dir" to "dst_dir"
pub fn install_all(src_dir: &str, dst_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    match fs::create_dir(src_dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }?;
    Ok(())
}

/// Copies the given files to "dst_dir"
pub fn install_some(src_files: &str, dst_dir: &str) {
    
}


#[cfg(test)]
mod tests {
    use std::{fs::{create_dir, remove_dir_all}, io::Write};

use super::*;

    #[test]
    fn copy_all_files_dst_dir_exists() {
        let src_dir = "/tmp/rust_test_src";
        match fs::remove_dir_all(src_dir) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }.expect("msg");

        // Create source directory with two files
        fs::create_dir(src_dir).expect("/tmp should exist");
        let mut file = fs::File::create(PathBuf::from(src_dir).join("file1.txt")).unwrap();
        file.write("file1 text\n".as_bytes()).unwrap();

        let mut file = fs::File::create(PathBuf::from(src_dir).join("file2.txt")).unwrap();
        file.write("file2 text\n".as_bytes()).unwrap();

        // Create dst_dir   
        let dst_dir = "/tmp/rust_test_dst";
        fs::remove_dir_all(dst_dir).expect("Destination tmp_dir should be fresh");
        fs::create_dir(dst_dir).expect("Ok");

        install_all(src_dir, dst_dir);

        panic!()
    }

    #[test]
    fn copy_all_files_dst_dir_not_exists() {
        panic!()
    }

    #[test]
    fn copy_all_files_src_dir_not_exists() {
        panic!()
    }

    #[test]
    fn copy_all_files_src_dir_is_empty() {
        panic!()
    }
}