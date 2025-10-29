use std::{
    fs::File,
    io::{BufReader, BufWriter, Error, Write},
    path::Path,
    process,
};

use crate::logentry::LogEntry;

struct WriteAheadLog<'a> {
    path: &'a Path,
}

impl<'a> WriteAheadLog<'a> {
    pub fn new(path: &'a str) -> Self {
        let p = Path::new(path);
        WriteAheadLog { path: p }
    }

    pub fn append(&self, log_entry: &LogEntry) -> Result<(), Error> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path)?;

        let mut writer = BufWriter::new(&file);
        let config = bincode::config::standard();

        bincode::encode_into_std_write(log_entry, &mut writer, config).unwrap_or_else(|err| {
            eprintln!("ENCODE ERROR: {err}");
            process::exit(1)
        });

        writer.flush()?;
        Ok(())
    }

    pub fn replay(&self) -> Result<Vec<LogEntry>, Error> {
        let file = File::open(self.path)?;

        let mut reader = BufReader::new(&file);
        let config = bincode::config::standard();

        let mut entries = Vec::new();

        while let Ok(entry) = bincode::decode_from_std_read(&mut reader, config) {
            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logentry::EntryType;
    use tempfile::TempDir;

    // Helper function to create a temporary test directory
    fn setup_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    // Helper function to create a test log entry
    fn create_test_entry(key: &[u8], value: &[u8], entry_type: EntryType) -> LogEntry {
        LogEntry {
            entry_type,
            key: key.to_vec(),
            value: value.to_vec(),
        }
    }

    #[test]
    fn test_new_wal_creates_instance() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        assert_eq!(wal.path, Path::new(log_path_str));
    }

    #[test]
    fn test_append_single_entry() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let entry = create_test_entry(b"key1", b"value1", EntryType::Put);

        let result = wal.append(&entry);
        assert!(result.is_ok());
        assert!(log_path.exists());
    }

    #[test]
    fn test_append_and_replay_single_entry() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let entry = create_test_entry(b"key1", b"value1", EntryType::Put);

        wal.append(&entry).unwrap();
        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], entry);
    }

    #[test]
    fn test_append_multiple_entries() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);

        let entry1 = create_test_entry(b"key1", b"value1", EntryType::Put);
        let entry2 = create_test_entry(b"key2", b"value2", EntryType::Put);
        let entry3 = create_test_entry(b"key3", b"", EntryType::Delete);

        wal.append(&entry1).unwrap();
        wal.append(&entry2).unwrap();
        wal.append(&entry3).unwrap();

        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], entry1);
        assert_eq!(entries[1], entry2);
        assert_eq!(entries[2], entry3);
    }

    #[test]
    fn test_replay_preserves_order() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);

        let entries: Vec<LogEntry> = (0..10)
            .map(|i| {
                create_test_entry(
                    format!("key{}", i).as_bytes(),
                    format!("value{}", i).as_bytes(),
                    EntryType::Put,
                )
            })
            .collect();

        for entry in &entries {
            wal.append(entry).unwrap();
        }

        let replayed = wal.replay().unwrap();
        assert_eq!(replayed.len(), entries.len());

        for (i, (original, replayed)) in entries.iter().zip(replayed.iter()).enumerate() {
            assert_eq!(original, replayed, "Entry {} doesn't match", i);
        }
    }

    #[test]
    fn test_replay_empty_log() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        // Create empty file
        File::create(&log_path).unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_replay_nonexistent_file() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("nonexistent.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let result = wal.replay();

        assert!(result.is_err());
    }

    #[test]
    fn test_append_with_empty_key() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let entry = create_test_entry(b"", b"value", EntryType::Put);

        wal.append(&entry).unwrap();
        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key.len(), 0);
    }

    #[test]
    fn test_append_with_empty_value() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let entry = create_test_entry(b"key", b"", EntryType::Put);

        wal.append(&entry).unwrap();
        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value.len(), 0);
    }

    #[test]
    fn test_append_with_binary_data() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let binary_key = vec![0u8, 1, 2, 255, 254, 128];
        let binary_value = vec![255u8, 0, 128, 64, 32];
        let entry = create_test_entry(&binary_key, &binary_value, EntryType::Put);

        wal.append(&entry).unwrap();
        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, binary_key);
        assert_eq!(entries[0].value, binary_value);
    }

    #[test]
    fn test_append_large_entry() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);
        let large_key = vec![b'k'; 10_000];
        let large_value = vec![b'v'; 100_000];
        let entry = create_test_entry(&large_key, &large_value, EntryType::Put);

        wal.append(&entry).unwrap();
        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key.len(), 10_000);
        assert_eq!(entries[0].value.len(), 100_000);
    }

    #[test]
    fn test_both_entry_types() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);

        let put_entry = create_test_entry(b"key1", b"value1", EntryType::Put);
        let delete_entry = create_test_entry(b"key2", b"", EntryType::Delete);

        wal.append(&put_entry).unwrap();
        wal.append(&delete_entry).unwrap();

        let entries = wal.replay().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].entry_type, EntryType::Put);
        assert_eq!(entries[1].entry_type, EntryType::Delete);
    }

    #[test]
    fn test_persistence_across_instances() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        // First instance - write data
        {
            let wal1 = WriteAheadLog::new(log_path_str);
            let entry = create_test_entry(b"key1", b"value1", EntryType::Put);
            wal1.append(&entry).unwrap();
        }

        // Second instance - read data
        {
            let wal2 = WriteAheadLog::new(log_path_str);
            let entries = wal2.replay().unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].key, b"key1");
        }
    }

    #[test]
    fn test_append_creates_file_if_not_exists() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("new_file.log");
        let log_path_str = log_path.to_str().unwrap();

        assert!(!log_path.exists());

        let wal = WriteAheadLog::new(log_path_str);
        let entry = create_test_entry(b"key1", b"value1", EntryType::Put);

        wal.append(&entry).unwrap();

        assert!(log_path.exists());
    }

    #[test]
    fn test_multiple_appends_to_same_file() {
        let temp_dir = setup_temp_dir();
        let log_path = temp_dir.path().join("test.log");
        let log_path_str = log_path.to_str().unwrap();

        let wal = WriteAheadLog::new(log_path_str);

        // Append first entry
        let entry1 = create_test_entry(b"key1", b"value1", EntryType::Put);
        wal.append(&entry1).unwrap();

        // Verify first entry
        let entries = wal.replay().unwrap();
        assert_eq!(entries.len(), 1);

        // Append second entry
        let entry2 = create_test_entry(b"key2", b"value2", EntryType::Put);
        wal.append(&entry2).unwrap();

        // Verify both entries
        let entries = wal.replay().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry1);
        assert_eq!(entries[1], entry2);
    }
}
