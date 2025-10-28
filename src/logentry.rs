use std::io::Error;

#[derive(Debug, PartialEq)]
pub enum EntryType {
    Put,
    Delete,
}

#[derive(Debug, PartialEq)]
pub struct LogEntry {
    entry_type: EntryType,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl LogEntry {
    pub fn new(entry_type: EntryType, key: &[u8], value: &[u8]) -> Self {
        LogEntry {
            entry_type,
            key: key.to_vec(),
            value: value.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_entry() {
        let entry = LogEntry::new(EntryType::Put, &[1, 2, 3], &[4, 5, 6]);
        assert_eq!(entry.entry_type, EntryType::Put);
        assert_eq!(entry.key, vec![1, 2, 3]);
        assert_eq!(entry.value, vec![4, 5, 6]);
    }

    #[test]
    fn test_delete_entry() {
        let entry = LogEntry::new(EntryType::Delete, &[42], &[]);
        assert_eq!(entry.entry_type, EntryType::Delete);
        assert_eq!(entry.key, vec![42]);
    }

    #[test]
    fn test_empty_key_value() {
        let entry = LogEntry::new(EntryType::Put, &[], &[]);
        assert_eq!(entry.key, Vec::<u8>::new());
        assert_eq!(entry.value, Vec::<u8>::new());
    }

    #[test]
    fn test_string_keys() {
        let entry = LogEntry::new(EntryType::Put, "hello".as_bytes(), "world".as_bytes());
        assert_eq!(entry.key, b"hello".to_vec());
        assert_eq!(entry.value, b"world".to_vec());
    }

    #[test]
    fn test_integer_keys() {
        let key = 42u64.to_le_bytes();
        let value = 100u64.to_le_bytes();
        let entry = LogEntry::new(EntryType::Put, &key, &value);

        assert_eq!(
            u64::from_le_bytes(entry.key.as_slice().try_into().unwrap()),
            42
        );
        assert_eq!(
            u64::from_le_bytes(entry.value.as_slice().try_into().unwrap()),
            100
        );
    }

    #[test]
    fn test_large_values() {
        let large_key = vec![0u8; 1000];
        let large_value = vec![255u8; 1000];
        let entry = LogEntry::new(EntryType::Put, &large_key, &large_value);

        assert_eq!(entry.key.len(), 1000);
        assert_eq!(entry.value.len(), 1000);
    }
}
