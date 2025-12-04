use std::{
    fs::{File, OpenOptions},
    io::{Error, Write},
    path::PathBuf,
};

use crc32fast::Hasher;

use crate::{
    errors::storage_errors::StorageError, file_manager, storage::internal_key::InternalKey,
};

#[derive(Debug)]
enum EntryType {
    Put,
    Delete,
}

impl From<EntryType> for u8 {
    fn from(value: EntryType) -> Self {
        match value {
            EntryType::Put => 0,
            EntryType::Delete => 1,
        }
    }
}

#[derive(Debug)]
pub struct WriteAheadLog {
    file: File,
    path: PathBuf,
    bytes_written: u64,
}

impl WriteAheadLog {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let file = OpenOptions::new().append(true).create(true).open(&path)?;
        Ok(WriteAheadLog {
            file,
            path,
            bytes_written: 0,
        })
    }

    pub fn write_put(&mut self, key: &InternalKey, value: &[u8]) -> Result<(), Error> {
        let k_bytes = key.encode();
        let k_len = k_bytes.len() as u32;
        let v_len = value.len() as u32;

        let mut hasher = Hasher::new();
        hasher.update(&[0u8]);
        hasher.update(&k_len.to_be_bytes());
        hasher.update(&k_bytes);
        hasher.update(&v_len.to_be_bytes());
        hasher.update(value);
        let crc = hasher.finalize();

        let mut entry_bytes = Vec::new();

        entry_bytes.push(0u8);
        entry_bytes.extend_from_slice(&k_len.to_be_bytes());
        entry_bytes.extend_from_slice(&k_bytes);
        entry_bytes.extend_from_slice(&v_len.to_be_bytes());
        entry_bytes.extend_from_slice(value);
        entry_bytes.extend_from_slice(&crc.to_be_bytes());

        self.file.write_all(&entry_bytes)?;

        self.file.sync_all()?;

        self.bytes_written += entry_bytes.len() as u64;

        Ok(())
    }

    pub fn write_delete(&mut self, key: &InternalKey) -> Result<(), Error> {
        let k_bytes = key.encode();
        let k_len = k_bytes.len() as u32;
        let v_len: u32 = 0;

        let mut hasher = Hasher::new();
        hasher.update(&[1u8]);
        hasher.update(&k_len.to_be_bytes());
        hasher.update(&k_bytes);
        hasher.update(&v_len.to_be_bytes());
        let crc = hasher.finalize();

        let mut entry_bytes = Vec::new();

        entry_bytes.push(1u8);
        entry_bytes.extend_from_slice(&k_len.to_be_bytes());
        entry_bytes.extend_from_slice(&k_bytes);
        entry_bytes.extend_from_slice(&v_len.to_be_bytes());
        entry_bytes.extend_from_slice(&crc.to_be_bytes());

        self.file.write_all(&entry_bytes)?;

        self.file.sync_all()?;

        self.bytes_written += entry_bytes.len() as u64;

        Ok(())
    }
}
