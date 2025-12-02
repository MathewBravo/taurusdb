use std::io::Error;

use crate::skiplist::{self, SkipList, SkipListIter};
use crate::storage::internal_key::InternalKey;

pub struct MemTable {
    skiplist: SkipList,
    size_bytes: usize,
    max_size: usize,
}

impl MemTable {
    pub fn new(max_size: usize) -> Self {
        let skiplist = SkipList::new();
        let size_bytes = 0;
        MemTable {
            skiplist,
            size_bytes,
            max_size,
        }
    }
    pub fn put(&mut self, key: InternalKey, value: Vec<u8>) -> Result<(), Error> {
        let key_size = key.encode().len();
        let value_size = value.len();
        let overhead = 64;

        self.skiplist.insert(key, value)?;

        self.size_bytes += key_size + value_size + overhead;

        Ok(())
    }
    pub fn get(&self, key: &InternalKey) -> Option<Vec<u8>> {
        self.skiplist.get(key)
    }
    pub fn delete(&mut self, key: InternalKey) -> bool {
        let value = self.get(&key);
        if let Some(value) = value {
            self.size_bytes -= key.encode().len() + value.len() + 64;
            return self.skiplist.delete(&key);
        }
        false
    }
    pub fn is_full(&self) -> bool {
        self.size_bytes >= self.max_size
    }
    pub fn size(&self) -> usize {
        self.size_bytes
    }
    pub fn iter(&self) -> SkipListIter {
        self.skiplist.iter()
    }
}
