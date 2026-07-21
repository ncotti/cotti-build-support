// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Nicolas Gabriel Cotti

//! # Install
//!
//! Functions mainly used for installing packages.

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tar::Archive;
use xz2::read::XzDecoder;
use zstd;

use crate::common;

/// Mimics the "install \<src\> \<dst\>" command from GNU. Copies all files from
/// "src_dir" to "dst_dir", but preserves the "src_dir" folder structure
///
/// src_dir follows these conditions:
/// * If it is a directory, all its contents and from its sub-directories
/// will be copied.
/// * If it doesn't exist or is empty, an error will be returned.
/// * It supports glob expansion of files. In that case, only the files that
/// match the glob will be copied.
///
/// dst_dir follows these conditions:
/// * If it doesn't exists, the path will be created.
/// * If it already exists, contents will be replaced, but not deleted.
pub fn install(src_dir: impl AsRef<Path>, dst_dir: impl AsRef<Path>) -> io::Result<()> {
    let src_dir = src_dir.as_ref();
    let dst_dir = dst_dir.as_ref();

    let src_files: Vec<PathBuf> = if src_dir.to_string_lossy().contains(['*', '?', '[']) {
        common::find(src_dir)
    } else if src_dir.is_file() {
        vec![PathBuf::from(src_dir)]
    } else if src_dir.is_dir() {
        common::find(src_dir.join("**/*"))
    } else {
        Vec::new()
    };

    if src_files.is_empty() {
        let e = std::io::Error::new(
            ErrorKind::InvalidFilename,
            format!("No source files to be installed at: {:?}", src_dir),
        );
        return Err(e);
    }

    fs::create_dir_all(dst_dir)?;

    for file in src_files {
        let dst_file = Path::new(dst_dir).join(file.file_name().unwrap_or_default());
        fs::copy(file, dst_file)?;
    }

    Ok(())
}

/// Mimics the "tar -xf" command from GNU. Decompresses the given tar file
/// into the output_dir.
///
/// The tar file may have any of the following extensions, from which the
/// function will infer the decompression algorithm:
/// * `.tar`
/// * `.tar.bz2 | .tbz2 | .tbz`
/// * `.tar.xz | .txz`
/// * `.tar.lzma`
/// * `.tar.gz | .tgz`
/// * `.tar.zst | .tzst`
pub fn untar(
    tar: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tar = tar.as_ref();
    let file = fs::File::open(tar)?;

    match tar.extension().and_then(|e| e.to_str()) {
        Some("tar") => {
            let mut archive = Archive::new(file);
            archive.unpack(output_dir)?;
        }
        Some("bz2") | Some("tbz2") | Some("tbz") => {
            let decoder = BzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(output_dir)?;
        }
        Some("xz") | Some("txz") => {
            let decoder = XzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(output_dir)?;
        }
        Some("lzma") => {
            let decoder = XzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(output_dir)?;
        }
        Some("gz") | Some("tgz") => {
            let decoder = GzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(output_dir)?;
        }
        Some("zst") | Some("tzst") => {
            let decoder = zstd::Decoder::new(file)?;
            let mut archive = Archive::new(decoder);
            archive.unpack(output_dir)?;
        }
        Some("zip") => {
            let mut zip = zip::ZipArchive::new(file)?;
            zip.extract(output_dir)?;
        }
        Some(_) | None => {
            let e = std::io::Error::new(
                ErrorKind::InvalidFilename,
                format!("Unknown tar extension: {:?}", tar),
            );
            return Err(Box::new(e));
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::common;
    use tempfile::{NamedTempFile, tempdir, tempdir_in};

    mod untar {
        use super::*;

        fn test_untar_generic(tar_file: &str) {
            let out_dir_parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("test_files");

            let out_dir = tempdir_in(&out_dir_parent).unwrap();

            let tar = out_dir_parent.join(tar_file);
            untar(tar, &out_dir).expect("Ok");
            assert!(
                fs::read(out_dir_parent.join("file1.txt")).unwrap()
                    == fs::read(out_dir.path().join("file1.txt")).unwrap()
            );
            assert!(
                fs::read(out_dir_parent.join("file2.txt")).unwrap()
                    == fs::read(out_dir.path().join("file2.txt")).unwrap()
            );
        }

        #[test]
        fn untar_tar() {
            test_untar_generic("files.tar");
        }

        #[test]
        fn untar_bzip2() {
            test_untar_generic("files.tar.bz2");
        }

        #[test]
        fn untar_gzip() {
            test_untar_generic("files.tar.gz");
        }

        #[test]
        fn untar_lzma() {
            test_untar_generic("files.tar.lzma");
        }

        #[test]
        fn untar_xz() {
            test_untar_generic("files.tar.xz");
        }

        #[test]
        fn untar_zst() {
            test_untar_generic("files.tar.zst");
        }

        #[test]
        fn untar_zip() {
            test_untar_generic("files.zip");
        }
    }

    mod install {
        use super::*;

        #[test]
        fn src_dir_not_exist() {
            let src_dir = tempdir().unwrap();
            let sub_src_dir = Path::new(src_dir.path()).join("not_a_dir");
            let dst_dir = tempdir().unwrap();

            // Files because the src_dir doesn't exist
            let e = install(&sub_src_dir, &dst_dir).unwrap_err();
            assert!(e.to_string().contains(sub_src_dir.to_str().unwrap()));

            // Files because the src_dir exists, but it is empty
            let e = install(&src_dir, &dst_dir).unwrap_err();
            assert!(e.to_string().contains(src_dir.path().to_str().unwrap()));
        }

        #[test]
        fn copy_all_files() {
            let src_dir = tempdir().unwrap();
            let src_files = [
                NamedTempFile::new_in(&src_dir).unwrap(),
                NamedTempFile::new_in(&src_dir).unwrap(),
            ];
            let dst_dir = tempdir().unwrap();

            for file in &src_files {
                assert!(file.path().exists());
                assert!(
                    !dst_dir
                        .path()
                        .join(file.path().file_name().unwrap())
                        .exists()
                );
            }
            install(&src_dir, &dst_dir).expect("Ok");
            for file in &src_files {
                assert!(file.path().exists());
                assert!(
                    dst_dir
                        .path()
                        .join(file.path().file_name().unwrap())
                        .exists()
                );
            }
        }

        #[test]
        fn copy_glob_files() {
            let src_dir = tempdir().unwrap();
            let sub_src_dir = tempdir_in(&src_dir).unwrap();
            let src_files_txt = [
                PathBuf::from(src_dir.path()).join("file1.txt"),
                PathBuf::from(src_dir.path()).join("file2.txt"),
                PathBuf::from(sub_src_dir.path()).join("file3.txt"),
            ];
            let src_files_json = [
                PathBuf::from(src_dir.path()).join("file4.json"),
                PathBuf::from(sub_src_dir.path()).join("file5.json"),
            ];

            let dst_dir = tempdir().unwrap();

            for file in src_files_txt.iter().chain(&src_files_json) {
                fs::File::create(file).expect("Ok");
            }

            for file in &src_files_json {
                assert!(file.exists());
                assert!(!dst_dir.path().join(file.file_name().unwrap()).exists());
            }

            assert!(common::find(dst_dir.path().join("**/*")).is_empty());
            install(src_dir.path().join("**/*.txt"), &dst_dir).expect("Ok");
            assert!(common::find(dst_dir.path().join("**/*.json")).is_empty());
            assert!(common::find(dst_dir.path().join("**/*.txt")).len() == src_files_txt.len());
        }

        #[test]
        fn copy_with_dst_and_src_in_the_same_path() {
            let src_dir = tempdir().unwrap();
            let sub_src_dir = tempdir_in(&src_dir).unwrap();
            let file_src = sub_src_dir.path().join("file.txt");
            let file_dst = src_dir.path().join("file.txt");
            fs::File::create(&file_src).expect("Ok");

            // Try to copy file from /a/b/c/file.txt to /a/b/file.txt
            assert!(file_src.exists());
            assert!(!file_dst.exists());
            install(&sub_src_dir, &src_dir).expect("ok");
            assert!(file_dst.exists());
        }
    }
}
