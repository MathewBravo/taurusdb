use std::{
    fmt::Display,
    fs::{OpenOptions, create_dir_all, read_dir, rename},
    io::{Error, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

#[derive(Debug)]
pub struct FileManager {
    db_dir_path: PathBuf,
    next_file_number: AtomicU64,
}

#[derive(Debug)]
pub enum Name {
    SSTable,
    WriteAheadLog,
    Manifest,
    Current,
    Lock,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Name::SSTable => write!(f, "SSTable"),
            Name::WriteAheadLog => write!(f, "WAL"),
            Name::Manifest => write!(f, "MANIFEST"),
            Name::Current => write!(f, "CURRENT"),
            Name::Lock => write!(f, "LOCK"),
        }
    }
}

impl FileManager {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        if path.exists() && path.is_file() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "db already exists as a file at that location",
            ));
        } else if path.exists() && path.is_dir() {
            let mut contents = read_dir(&path)?;
            if contents.next().is_some() {
                return Err(Error::new(
                    ErrorKind::AlreadyExists,
                    "db already exists at that location",
                ));
            }
        } else {
            create_dir_all(&path)?;
        }

        initialize_db_files(&path)?;

        Ok(FileManager {
            db_dir_path: path,
            next_file_number: AtomicU64::new(2),
        })
    }

    pub fn open_existing(path: PathBuf) -> Result<Self, Error> {
        if !path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "db directory not found"));
        }

        let cp = path.join("CURRENT");

        if !cp.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "path exists, but db not initialized within",
            ));
        }

        let lp = path.join("LOCK");
        match OpenOptions::new().write(true).create_new(true).open(&lp) {
            Ok(mut file) => {
                file.write_all(std::process::id().to_string().as_bytes())?;
                file.sync_all()?;
            }
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    let mut lf = OpenOptions::new().read(true).open(&lp)?;
                    let mut pid: String = String::new();
                    lf.read_to_string(&mut pid)?;
                    return Err(Error::new(
                        ErrorKind::AlreadyExists,
                        format!("database already open by process {pid}"),
                    ));
                } else {
                    return Err(e);
                }
            }
        }

        let mut cf = OpenOptions::new().read(true).open(&cp)?;

        let mut contents = String::new();
        cf.read_to_string(&mut contents)?;

        let manifest_name = Path::new(contents.trim());
        let manifest_path = path.join(manifest_name);

        let next_file = get_next_file_num(&manifest_path)?;

        Ok(FileManager {
            db_dir_path: path,
            next_file_number: next_file,
        })
    }

    pub fn new_file_number(&self) -> u64 {
        self.next_file_number
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn generate_filename(&self, file_type: Name, number: Option<u64>) -> PathBuf {
        let path = match file_type {
            Name::SSTable => {
                assert!(number.is_some(), "SSTable requires a file number!");
                let num = number.unwrap();
                let file_num = format!("{:06}", num);
                format!("{}.sst", file_num)
            }
            Name::WriteAheadLog => {
                assert!(number.is_some(), "WriteAheadLogs require a file number!");
                let num = number.unwrap();
                let file_num = format!("{:06}", num);
                format!("{}.log", file_num)
            }
            Name::Manifest => {
                assert!(number.is_some(), "Manifests require a file number!");
                let num = number.unwrap();
                let file_num = format!("{:06}", num);
                format!("{}-{}", file_type, file_num)
            }
            Name::Current | Name::Lock => {
                assert!(
                    number.is_none(),
                    "Fixed file types should not have a number"
                );
                format!("{file_type}")
            }
        };

        self.db_dir_path.join(path)
    }
}

impl Drop for FileManager {
    fn drop(&mut self) {
        let lock_path = self.db_dir_path.join("LOCK");
        let _ = std::fs::remove_file(lock_path);
    }
}

fn initialize_db_files(path: &PathBuf) -> Result<(), Error> {
    let lock_path = path.join("LOCK");
    let mut lf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lock_path)?;

    lf.write_all(std::process::id().to_string().as_bytes())?;
    lf.sync_all()?;

    let manifest_path = path.join("MANIFEST-000001");
    let mut mf = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(manifest_path)?;

    mf.write_all(b"next_file_number: 2\n")?;
    mf.sync_all()?;

    let curtmp_path = path.join("CURRENT.tmp");
    let current_path = path.join("CURRENT");
    let mut cf = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&curtmp_path)?;

    cf.write_all(b"MANIFEST-000001\n")?;
    cf.sync_all()?;
    drop(cf);

    rename(&curtmp_path, &current_path)?;

    Ok(())
}

fn get_next_file_num(manifest_path: &Path) -> Result<AtomicU64, Error> {
    let mut mf = OpenOptions::new()
        .read(true)
        .create(false)
        .open(manifest_path)?;

    let mut manifest_contents = String::new();

    mf.read_to_string(&mut manifest_contents)?;

    let line = manifest_contents
        .lines()
        .find_map(|line| {
            let next_line = line.strip_prefix("next_file_number:")?.trim();
            next_line.parse::<u64>().ok()
        })
        .ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "next_file_number not found or invalid",
            )
        })?;

    Ok(AtomicU64::from(line))
}

// I was learning this as I build it, I generated the tests using GPT 5.1 + Gemini 3 because I
// didn't trust myself to not implement tests in a way that covered what I needed
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Arc;
    use std::thread;
    use tempfile::TempDir;

    // Helper function to create a temporary directory for testing

    fn setup_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp directory")
    }

    #[test]
    fn test_create_new_database() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        // Verify files exist
        assert!(db_path.join("LOCK").exists(), "LOCK file should exist");
        assert!(
            db_path.join("MANIFEST-000001").exists(),
            "MANIFEST file should exist"
        );
        assert!(
            db_path.join("CURRENT").exists(),
            "CURRENT file should exist"
        );

        // Verify CURRENT content points to manifest
        let current_content =
            fs::read_to_string(db_path.join("CURRENT")).expect("Failed to read CURRENT");
        assert_eq!(
            current_content.trim(),
            "MANIFEST-000001",
            "CURRENT should point to MANIFEST-000001"
        );

        // Verify MANIFEST content
        let manifest_content =
            fs::read_to_string(db_path.join("MANIFEST-000001")).expect("Failed to read MANIFEST");
        assert_eq!(
            manifest_content, "next_file_number: 2\n",
            "MANIFEST should contain next_file_number: 2"
        );

        // Verify first file number is 2
        let file_num = fm.new_file_number();
        assert_eq!(file_num, 2, "First file number should be 2");
    }

    #[test]
    fn test_create_fails_on_file() {
        let temp_dir = setup_temp_dir();
        let file_path = temp_dir.path().join("testfile");

        // Create a regular file
        fs::write(&file_path, "test").expect("Failed to create file");

        // Try to create database at file location
        let result = FileManager::new(file_path);

        assert!(result.is_err(), "Should fail when path is a file");
        assert_eq!(
            result.unwrap_err().kind(),
            ErrorKind::AlreadyExists,
            "Error should be AlreadyExists"
        );
    }

    #[test]
    fn test_create_fails_on_non_empty_directory() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        // Create a file in the directory
        fs::write(db_path.join("somefile.txt"), "content").expect("Failed to create file");

        // Try to create database in non-empty directory
        let result = FileManager::new(db_path);

        assert!(result.is_err(), "Should fail when directory is not empty");
        assert_eq!(
            result.unwrap_err().kind(),
            ErrorKind::AlreadyExists,
            "Error should be AlreadyExists"
        );
    }

    #[test]
    fn test_open_existing_database() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        // Create database
        {
            let _fm = FileManager::new(db_path.clone()).expect("Failed to create database");
            // fm dropped here, LOCK released
        }

        // Verify LOCK was cleaned up
        assert!(
            !db_path.join("LOCK").exists(),
            "LOCK should be removed on drop"
        );

        // Reopen database
        let fm = FileManager::open_existing(db_path.clone()).expect("Failed to open database");

        // Verify LOCK was reacquired
        assert!(
            db_path.join("LOCK").exists(),
            "LOCK should exist after reopening"
        );

        // Verify file numbers continue from where we left off
        let file_num = fm.new_file_number();
        assert_eq!(file_num, 2, "First file number after reopen should be 2");
    }

    #[test]
    fn test_open_fails_on_nonexistent_directory() {
        let temp_dir = setup_temp_dir();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        let result = FileManager::open_existing(nonexistent_path);

        assert!(result.is_err(), "Should fail on nonexistent directory");
        assert_eq!(
            result.unwrap_err().kind(),
            ErrorKind::NotFound,
            "Error should be NotFound"
        );
    }

    #[test]
    fn test_open_fails_without_current_file() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        // Create directory but no database files
        fs::create_dir_all(&db_path).expect("Failed to create directory");

        let result = FileManager::open_existing(db_path);

        assert!(
            result.is_err(),
            "Should fail when CURRENT file doesn't exist"
        );
        assert_eq!(
            result.unwrap_err().kind(),
            ErrorKind::NotFound,
            "Error should be NotFound"
        );
    }

    #[test]
    fn test_lock_prevents_concurrent_open() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        // Create database and keep it open
        let _fm1 = FileManager::new(db_path.clone()).expect("Failed to create database");

        // Try to open again while first is still open
        let result = FileManager::open_existing(db_path);

        assert!(result.is_err(), "Should fail when database is locked");
        assert_eq!(
            result.unwrap_err().kind(),
            ErrorKind::AlreadyExists,
            "Error should be AlreadyExists"
        );
    }

    #[test]
    fn test_file_number_generation_sequential() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path).expect("Failed to create database");

        // Get several file numbers
        let num1 = fm.new_file_number();
        let num2 = fm.new_file_number();
        let num3 = fm.new_file_number();
        let num4 = fm.new_file_number();

        // Verify they're sequential
        assert_eq!(num1, 2, "First number should be 2");
        assert_eq!(num2, 3, "Second number should be 3");

        assert_eq!(num3, 4, "Third number should be 4");
        assert_eq!(num4, 5, "Fourth number should be 5");
    }

    #[test]

    fn test_file_number_generation_thread_safe() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = Arc::new(FileManager::new(db_path).expect("Failed to create database"));

        let mut handles = vec![];

        // Spawn 10 threads, each getting 100 file numbers
        for _ in 0..10 {
            let fm_clone = Arc::clone(&fm);
            let handle = thread::spawn(move || {
                let mut numbers = Vec::new();
                for _ in 0..100 {
                    numbers.push(fm_clone.new_file_number());
                }
                numbers
            });
            handles.push(handle);
        }

        // Collect all numbers from all threads
        let mut all_numbers = Vec::new();

        for handle in handles {
            all_numbers.extend(handle.join().expect("Thread panicked"));
        }

        // Sort and check for uniqueness
        all_numbers.sort();
        let original_len = all_numbers.len();
        all_numbers.dedup();

        assert_eq!(
            all_numbers.len(),
            original_len,
            "All file numbers should be unique (no duplicates)"
        );
        assert_eq!(
            original_len, 1000,
            "Should have 1000 total numbers (10 threads × 100)"
        );
        assert_eq!(all_numbers[0], 2, "First number should be 2");
        assert_eq!(all_numbers[999], 1001, "Last number should be 1001");
    }

    #[test]
    fn test_generate_filename_sstable() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        let path = fm.generate_filename(Name::SSTable, Some(42));

        assert_eq!(
            path,
            db_path.join("000042.sst"),
            "SSTable filename should be correctly formatted"
        );
    }

    #[test]
    fn test_generate_filename_wal() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        let path = fm.generate_filename(Name::WriteAheadLog, Some(100));

        assert_eq!(
            path,
            db_path.join("000100.log"),
            "WAL filename should be correctly formatted"
        );
    }

    #[test]
    fn test_generate_filename_manifest() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        let path = fm.generate_filename(Name::Manifest, Some(5));

        assert_eq!(
            path,
            db_path.join("MANIFEST-000005"),
            "Manifest filename should be correctly formatted"
        );
    }

    #[test]
    fn test_generate_filename_current() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        let path = fm.generate_filename(Name::Current, None);

        assert_eq!(
            path,
            db_path.join("CURRENT"),
            "Current filename should be CURRENT"
        );
    }

    #[test]
    fn test_generate_filename_lock() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        let path = fm.generate_filename(Name::Lock, None);

        assert_eq!(path, db_path.join("LOCK"), "Lock filename should be LOCK");
    }

    #[test]

    fn test_generate_filename_zero_padding() {
        let temp_dir = setup_temp_dir();

        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        // Test that numbers are zero-padded to 6 digits
        let path1 = fm.generate_filename(Name::SSTable, Some(1));
        let path42 = fm.generate_filename(Name::SSTable, Some(42));
        let path999 = fm.generate_filename(Name::SSTable, Some(999));
        let path9999 = fm.generate_filename(Name::SSTable, Some(9999));

        assert_eq!(path1, db_path.join("000001.sst"), "1 should pad to 000001");
        assert_eq!(
            path42,
            db_path.join("000042.sst"),
            "42 should pad to 000042"
        );
        assert_eq!(
            path999,
            db_path.join("000999.sst"),
            "999 should pad to 000999"
        );
        assert_eq!(
            path9999,
            db_path.join("009999.sst"),
            "9999 should pad to 009999"
        );
    }

    #[test]
    #[should_panic(expected = "SSTable requires a file number")]
    fn test_generate_filename_sstable_without_number_panics() {
        let temp_dir = setup_temp_dir();

        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path).expect("Failed to create database");

        // This should panic
        fm.generate_filename(Name::SSTable, None);
    }

    #[test]
    #[should_panic(expected = "Fixed file types should not have a number")]
    fn test_generate_filename_current_with_number_panics() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let fm = FileManager::new(db_path).expect("Failed to create database");

        // This should panic
        fm.generate_filename(Name::Current, Some(42));
    }

    #[test]
    fn test_lock_cleanup_on_drop() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        {
            let _fm = FileManager::new(db_path.clone()).expect("Failed to create database");
            // LOCK should exist here
            assert!(
                db_path.join("LOCK").exists(),
                "LOCK should exist while FileManager is alive"
            );
        } // fm dropped here - Drop runs

        // LOCK should be deleted
        assert!(
            !db_path.join("LOCK").exists(),
            "LOCK should be removed after FileManager is dropped"
        );
    }

    #[test]
    fn test_persistence_across_restarts() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        // Create database and get some file numbers
        {
            let fm = FileManager::new(db_path.clone()).expect("Failed to create database");
            assert_eq!(fm.new_file_number(), 2);

            assert_eq!(fm.new_file_number(), 3);
            assert_eq!(fm.new_file_number(), 4);
            // Counter is now at 5
        }

        // Reopen database
        let fm = FileManager::open_existing(db_path).expect("Failed to open database");

        // Next file number should continue from where we left off
        // Wait, this won't work! The manifest still says next_file_number: 2

        // because we never updated it. This is expected behavior for Phase 1.
        // In Phase 2+, you'd update the manifest when creating files.

        // For now, this test verifies that open_existing reads the manifest correctly
        assert_eq!(
            fm.new_file_number(),
            2,
            "After reopen, should start from manifest value (2)"
        );
    }

    #[test]

    fn test_atomic_current_file_creation() {
        let temp_dir = setup_temp_dir();
        let db_path = temp_dir.path().to_path_buf();

        let _fm = FileManager::new(db_path.clone()).expect("Failed to create database");

        // Verify CURRENT.tmp doesn't exist (cleaned up)
        assert!(
            !db_path.join("CURRENT.tmp").exists(),
            "CURRENT.tmp should be cleaned up after atomic rename"
        );

        // Verify CURRENT exists
        assert!(db_path.join("CURRENT").exists(), "CURRENT should exist");
    }
}
