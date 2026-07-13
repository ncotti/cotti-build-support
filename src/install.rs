use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::Path;

use crate::common;

/// Copies all files from "src_dir" to "dst_dir"
/// If "dst_dir" already exists, all its contents will be erased.
/// If "src_dir" doesn't exist or is an empty folder, an error will be returned.
/// If "src_dir" is a directory, all its contents recursevely will be copied.
/// If "src_dir" is a glob pattern, only the files that match that pattern will
/// be copied. If none match, an error will be issued.
pub fn install(src_dir: impl AsRef<Path>, dst_dir: impl AsRef<Path>) -> io::Result<()> {
    let src_dir = src_dir.as_ref();
    let dst_dir = dst_dir.as_ref();

    let src_files = if src_dir.to_string_lossy().contains(['*', '?', '[']) {
        // If src_dir is a glob, find all files as-is
        common::find(src_dir)
    } else {
        // If it is not a glob, find all files
        common::find(src_dir.join("**/*"))
    };

    if src_files.is_empty() {
        let e = std::io::Error::new(
            ErrorKind::InvalidFilename,
            format!("No source files to be installed at: {:?}", src_dir),
        );
        return Err(e);
    }

    common::rm_rf(dst_dir)?;
    fs::create_dir_all(dst_dir)?;

    for file in src_files {
        let dst_file = Path::new(dst_dir).join(file.file_name().unwrap_or_default());
        fs::copy(file, dst_file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::common;
    use std::path::PathBuf;

    mod install {
        use tempfile::{NamedTempFile, tempdir, tempdir_in};

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
    }
}
