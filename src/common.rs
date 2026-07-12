use std::{fs, io::{self, ErrorKind}, path::{Path, PathBuf}};

use glob;

/// Mimics the "rm -rf" from Unix, i.e., delete the given path, either a file
/// or a folder, recursively and don't fail if the path doesn't exist.
pub fn rm_rf(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();

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

/// Returns the list of files and folders that match the given "file",
/// with glob expansion. If no file is found, an empty vector is returned.
pub fn find(file: impl AsRef<Path>) -> Vec<PathBuf> {
    let pattern = file.as_ref().to_string_lossy();

    glob::glob(&pattern)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .collect()
}

/// Returns the list of files, not folders, that match the given "file",
/// with glob expansion. If no file is found, an empty vector is returned.
pub fn find_files(file: impl AsRef<Path>) -> Vec<PathBuf> {
    find(file)
        .into_iter()
        .filter(|file| file.is_file())
        .collect()
}

/// Returns the list of folders, not files that match the given "file",
/// with glob expansion. If no folder is found, an empty vector is returned.
pub fn find_dirs(file: impl AsRef<Path>) -> Vec<PathBuf> {
    find(file)
        .into_iter()
        .filter(|file| file.is_dir())
        .collect()
}

/// Forcefully creates a file and all the parent directories required to
/// reach that file. If the file already exists, it is truncated.
pub fn create_file(file: impl AsRef<Path>) -> io::Result<()> {
    let file = file.as_ref();

    if let Some(parent) = file.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    fs::File::create(file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
use std::io::Write;

use super::*;

    mod rm {
        use super::*;
        #[test]
        fn remove_file() {
            let path = "/tmp/test_file.txt";
            let mut file = fs::File::create("/tmp/test_file.txt").unwrap();
            file.write("some text\n".as_bytes()).unwrap();

            rm_rf(path).expect("Ok");
            assert!(! Path::new(path).exists());
        }

        #[test]
        fn remove_folder() {
            let path = Path::new("/tmp/test_folder");

            if ! path.exists() {
                fs::create_dir(path).expect("Path should be created.")
            }
            rm_rf(path).expect("Ok");
            assert!( ! path.exists());
        }

        #[test]
        fn remove_folder_recurse() {
            let dir1 = Path::new("/tmp/dir1");
            let dir2 = Path::new("/tmp/dir1/dir2");

            if ! dir1.exists() {
                fs::create_dir(dir1).expect("Path should be created.")
            }

            if ! dir2.exists() {
                fs::create_dir(dir2).expect("Path should be created.")
            }

            // Remove only second dir. First one should still be valid
            rm_rf(dir1).expect("Ok");
            assert!(! dir2.exists());
            assert!(! dir2.exists());
        }

        #[test]
        fn remove_path_not_exists() {
        let path = Path::new("/tmp/not_existing_folder");
        if path.exists() {
                fs::remove_dir_all(path).expect("Should be removed");
        }

        assert!(! path.exists());
        rm_rf(path).expect("Ok");
        assert!(! path.exists());
        }

        #[test]
        fn forbidden_remove() {
            let path = Path::new("/root");
            assert!(path.exists());
            rm_rf(path).expect_err("OS error for lack of permissions");
        }
    }

    mod find {
        use std::iter::zip;

use super::*;

        #[test]
        fn find_specific_file() {
            let file_path = "/tmp/example_dir/file1.txt";
            rm_rf(file_path).expect("Ok");
            fs::create_dir_all("/tmp/example_dir").expect("Ok");

            let found_files = find(file_path);
            println!("{found_files:?}");
            assert!(found_files.is_empty());

            fs::File::create(file_path).expect("Ok");

            let found_files = find(file_path);

            assert!(found_files.len() == 1);
            assert!(found_files[0] == PathBuf::from(file_path));
        }

        #[test]
        fn find_glob_files() {
            let parent_dir = "/tmp/example_dir";
            let files = [
                format!("{parent_dir}/file1.txt"),
                format!("{parent_dir}/file2.txt"),
                format!("{parent_dir}/file3.txt"),
                format!("{parent_dir}/file4.json"),
                format!("{parent_dir}/config.json"),
            ];

            let mut expected_txt_files: Vec<PathBuf> = Vec::new();
            expected_txt_files.push(PathBuf::from(&files[0]));
            expected_txt_files.push(PathBuf::from(&files[1]));
            expected_txt_files.push(PathBuf::from(&files[2]));
            expected_txt_files.sort();

            let mut expected_json_files: Vec<PathBuf> = Vec::new();
            expected_json_files.push(PathBuf::from(&files[3]));
            expected_json_files.push(PathBuf::from(&files[4]));
            expected_json_files.sort();

            let mut expected_prefixed_files: Vec<PathBuf> = Vec::new();
            expected_prefixed_files.push(PathBuf::from(&files[0]));
            expected_prefixed_files.push(PathBuf::from(&files[1]));
            expected_prefixed_files.push(PathBuf::from(&files[2]));
            expected_prefixed_files.push(PathBuf::from(&files[3]));
            expected_prefixed_files.sort();

            let mut expected_all_files: Vec<PathBuf> = Vec::new();
            for file in &files {
                expected_all_files.push(PathBuf::from(file));
            }
            expected_all_files.sort();


            fs::create_dir_all(parent_dir).expect("Ok");
            for file in &files {
                fs::File::create(file).expect("Ok");
            }

            // Glob for all files in folder
            let found_files = find(format!("{parent_dir}/*"));
            assert!(expected_all_files.len() == found_files.len());
            for (exp, actual) in zip(expected_all_files, found_files) {
                assert!(exp == actual);
            }

            let found_files = find(format!("{parent_dir}/*.txt"));
            assert!(expected_txt_files.len() == found_files.len());
            for (exp, actual) in zip(expected_txt_files, found_files) {
                assert!(exp == actual);
            }

            let found_files = find(format!("{parent_dir}/*.json"));
            assert!(expected_json_files.len() == found_files.len());
            for (exp, actual) in zip(expected_json_files, found_files) {
                assert!(exp == actual);
            }

            let found_files = find(format!("{parent_dir}/file*"));
            assert!(expected_prefixed_files.len() == found_files.len());
            for (exp, actual) in zip(expected_prefixed_files, found_files) {
                assert!(exp == actual);
            }

        }

        #[test]
        fn find_folders() {
            let dir = "/tmp/example_dir";
            fs::create_dir_all(dir).expect("Ok");

            let dirs = find(dir);
            assert!(dirs.len() == 1);
            assert!(dirs[0] == PathBuf::from(dir));
        }

        #[test]
        fn find_files_and_folders_mixed() {
            let dirs = [
                "/tmp/example_dir",
                "/tmp/example_dir/dir1",
                "/tmp/example_dir/dir2",
            ];

            let files = [
                format!("{}/file1.txt", dirs[0]),
                format!("{}/file2.txt", dirs[0]),
            ];

            rm_rf(dirs[0]).expect("Ok");
            for dir in &dirs {
                fs::create_dir_all(dir).expect("Ok");
            }

            for file in &files {
                fs::File::create(file).expect("Ok");
            }

            // Should find all files and folders
            let found_files = find(format!("{}/*", dirs[0]));
            println!("{:?}", found_files);
            assert!(found_files.len() == 4);
            assert!(found_files[0].to_str().unwrap() == dirs[1]);
            assert!(found_files[1].to_str().unwrap() == dirs[2]);
            assert!(found_files[2].to_str().unwrap() == files[0]);
            assert!(found_files[3].to_str().unwrap() == files[1]);

            // Should only find files
            let found_files = find_files(format!("{}/*", dirs[0]));
            assert!(found_files.len() == 2);
            assert!(found_files[0].to_str().unwrap() == files[0]);
            assert!(found_files[1].to_str().unwrap() == files[1]);

            // Should only find folders
            let found_files = find_dirs(format!("{}/*", dirs[0]));
            assert!(found_files.len() == 2);
            assert!(found_files[0].to_str().unwrap() == dirs[1]);
            assert!(found_files[1].to_str().unwrap() == dirs[2]);
        }

        #[test]
        fn find_glob_folders() {
            panic!();
        }
    }

    
}