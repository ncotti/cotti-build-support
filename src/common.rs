// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Nicolas Gabriel Cotti

//! # Common
//!
//! Functions that mimic the implementation of well-known Bash commands.

use glob;
use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

/// Mimics the "rm -rf" command from GNU.
///
/// Deletes the given path, either a file or a folder, recursively, and
/// doesn't fail if the path doesn't exist.
///
/// It does not support glob expansion, so it will return with an error if an
/// asterisk is part of the input path.
pub fn rm_rf(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();

    if path.to_string_lossy().contains(['*', '?', '[']) {
        let e = std::io::Error::new(
            ErrorKind::InvalidFilename,
            format!("Path to be removed should not be a glob: {:?}", path),
        );
        return Err(e);
    }

    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Mimics the "find -wholename \<pattern\>" command from GNU.
/// Returns the list of files and folders that match the given "pattern".
/// If "pattern" is a directory, then it will search for all files in it.
///
/// It supports glob expansion. If no file is found, an empty vector is returned.
pub fn find(pattern: impl AsRef<Path>) -> Vec<PathBuf> {
    let pattern = pattern.as_ref();

    let pattern = if pattern.is_dir() {
        pattern.join("*")
    } else {
        pattern.to_path_buf()
    };

    let pattern = pattern.to_string_lossy();

    glob::glob(&pattern)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .collect()
}

/// Mimics the "find -type f -wholename \<pattern\>" command from GNU.
/// Returns the list of files, not folders, that match the given "pattern".
/// If "pattern" is a directory, then it will search for all files in it.
///
/// It supports glob expansion. If no file is found, an empty vector is returned.
pub fn find_files(pattern: impl AsRef<Path>) -> Vec<PathBuf> {
    find(pattern)
        .into_iter()
        .filter(|file| file.is_file())
        .collect()
}
/// Mimics the "find -type d -wholename \<pattern\>" command from GNU.
/// Returns the list of folders, not files, that match the given "pattern".
/// If "pattern" is a directory, then it will search for all folders in it.
///
/// It supports glob expansion. If no file is found, an empty vector is returned.
pub fn find_dirs(pattern: impl AsRef<Path>) -> Vec<PathBuf> {
    find(pattern)
        .into_iter()
        .filter(|file| file.is_dir())
        .collect()
}

#[cfg(test)]
mod tests {
    use tempfile::{NamedTempFile, tempdir, tempdir_in};

    use super::*;

    mod rm {
        use super::*;
        #[test]
        fn remove_file() {
            let file = NamedTempFile::new().unwrap();
            let path = Path::new(file.path());
            assert!(path.exists());
            rm_rf(&file).expect("Ok");
            assert!(!path.exists());
        }

        #[test]
        fn remove_folder() {
            let dir = tempdir().unwrap();
            let path = Path::new(dir.path());
            assert!(path.exists());
            rm_rf(&dir).expect("Ok");
            assert!(!path.exists());
        }

        #[test]
        fn remove_folder_recurse() {
            let dir1 = tempdir().unwrap();
            let dir2 = tempdir_in(&dir1).unwrap();
            let path1 = Path::new(dir1.path());
            let path2 = Path::new(dir2.path());

            assert!(path1.exists());
            assert!(path2.exists());
            rm_rf(&dir1).expect("Ok");
            assert!(!path1.exists());
            assert!(!path2.exists());
        }

        #[test]
        fn remove_path_not_exists() {
            let dir = tempdir().unwrap();
            let path = Path::new(dir.path());
            assert!(path.exists());
            rm_rf(&dir).expect("OK");
            assert!(!path.exists());
            rm_rf(path).expect("Ok");
            assert!(!path.exists());
        }

        #[test]
        fn forbidden_remove() {
            let path = Path::new("/root");
            assert!(path.exists());
            rm_rf(path).expect_err("OS error for lack of permissions");
        }

        #[test]
        fn do_not_remove_globs() {
            let path = Path::new("/tmp/*");
            rm_rf(path).expect_err("Not globs");
        }
    }

    mod find {
        use super::*;
        use std::iter::zip;

        #[test]
        fn find_specific_file() {
            let file = NamedTempFile::new().unwrap();
            let path = Path::new(file.path());

            let found_files = find(&file);
            assert!(found_files.len() == 1);
            assert!(found_files[0].as_path() == path);

            rm_rf(&file).expect("Ok");
            let found_files = find(&file);
            assert!(found_files.is_empty());
        }

        #[test]
        fn find_glob_files() {
            let parent_dir = tempdir().unwrap();
            let parent_path = Path::new(parent_dir.path());

            let files = [
                PathBuf::from(parent_path).join("file1.txt"),
                PathBuf::from(parent_path).join("file2.txt"),
                PathBuf::from(parent_path).join("file3.txt"),
                PathBuf::from(parent_path).join("file4.json"),
                PathBuf::from(parent_path).join("config.json"),
            ];
            let mut files_sorted = files.clone();
            files_sorted.sort();

            let mut expected_txt_files: Vec<&PathBuf> = vec![&files[0], &files[1], &files[2]];
            expected_txt_files.sort();

            let mut expected_json_files: Vec<&PathBuf> = vec![&files[3], &files[4]];
            expected_json_files.sort();

            let mut expected_prefixed_files: Vec<&PathBuf> =
                vec![&files[0], &files[1], &files[2], &files[3]];
            expected_prefixed_files.sort();

            for file in &files {
                fs::File::create(file).expect("Ok");
            }

            // Glob for all files in folder
            let found_files = find(parent_path);
            assert!(files.len() == found_files.len());
            for (exp, actual) in zip(&files_sorted, found_files) {
                assert!(*exp == actual);
            }

            let found_files = find(parent_path.join("*.txt"));
            assert!(expected_txt_files.len() == found_files.len());
            for (exp, actual) in zip(expected_txt_files, found_files) {
                assert!(*exp == actual);
            }

            let found_files = find(parent_path.join("*.json"));
            assert!(expected_json_files.len() == found_files.len());
            for (exp, actual) in zip(expected_json_files, found_files) {
                assert!(*exp == actual);
            }

            let found_files = find(parent_path.join("file*"));
            assert!(expected_prefixed_files.len() == found_files.len());
            for (exp, actual) in zip(expected_prefixed_files, found_files) {
                assert!(*exp == actual);
            }
        }

        #[test]
        fn find_files_and_folders_mixed() {
            let parent_dir = tempdir().unwrap();
            let _dirs = [
                tempdir_in(&parent_dir).unwrap(),
                tempdir_in(&parent_dir).unwrap(),
            ];

            let _files = [
                NamedTempFile::new_in(&parent_dir).unwrap(),
                NamedTempFile::new_in(&parent_dir).unwrap(),
                NamedTempFile::new_in(&parent_dir).unwrap(),
            ];

            // Should find all files and folders
            let found_files = find(parent_dir.path());
            println!("{:?}", found_files);
            assert!(found_files.len() == 5);

            // Should only find files
            let found_files = find_files(parent_dir.path());
            assert!(found_files.len() == 3);

            // Should only find folders
            let found_files = find_dirs(parent_dir.path());
            assert!(found_files.len() == 2);
        }

        #[test]
        fn find_glob_folders() {
            let parent_dir1 = tempdir().unwrap();
            let parent_dir2 = tempdir_in(&parent_dir1).unwrap();
            let parent_dir3 = tempdir_in(&parent_dir1).unwrap();

            fs::File::create(parent_dir1.path().join("file1.txt")).expect("Ok");
            fs::File::create(parent_dir2.path().join("file2.txt")).expect("Ok");
            fs::File::create(parent_dir3.path().join("file3.txt")).expect("Ok");

            // file1.txt, file2.txt, file3.txt
            let found_files = find(parent_dir1.path().join("**/*.txt"));
            assert!(found_files.len() == 3);

            // 3 text files, plus 2 folders
            let found_files = find(parent_dir1.path().join("**/*"));
            assert!(found_files.len() == 5);
        }
    }
}
