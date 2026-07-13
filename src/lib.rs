#![warn(missing_docs)]

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use tar::Archive;
use std::fs;

pub mod install;
pub mod common;



/// Decompresses the given tar file into the output_dir.
/// The tar file may have any of the following extensions:
/// * .tar
pub fn untar(tar: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(tar)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(output_dir)?;
    Ok(())
}



pub fn install_ftd2xx(tar: &str, lib_install_dir: &str, header_install_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::copy("/home/nicolas.cotti/cotti/rust/cotti-build-support/tmp/linux-x86_64/libftd2xx.so", PathBuf::from(lib_install_dir).join("libftd2xx.so"))?;
    fs::copy("/home/nicolas.cotti/cotti/rust/cotti-build-support/tmp/linux-x86_64/libftd2xx.a", PathBuf::from(lib_install_dir).join("libftd2xx.a"))?;
    fs::copy("/home/nicolas.cotti/cotti/rust/cotti-build-support/tmp/linux-x86_64/ftd2xx.h", PathBuf::from(header_install_dir).join("ftd2xx.h"))?;
    fs::copy("/home/nicolas.cotti/cotti/rust/cotti-build-support/tmp/linux-x86_64/WinTypes.h", PathBuf::from(header_install_dir).join("WinTypes.h"))?;

    fs::remove_dir_all("tmp")?;
    Ok(())
}
