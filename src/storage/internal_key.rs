use core::cmp::Ordering;

use crate::errors::storage_errors::StorageError;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
#[repr(u8)]
pub enum KeyType {
    Delete,
    Put,
}

impl TryFrom<u8> for KeyType {
    type Error = StorageError;

    fn try_from(value: u8) -> Result<Self, StorageError> {
        match value {
            0 => Ok(KeyType::Delete),
            1 => Ok(KeyType::Put),
            _ => Err(StorageError::DecodeError(String::from(
                "could not parse key type from last byte",
            ))),
        }
    }
}

impl From<KeyType> for u8 {
    fn from(value: KeyType) -> Self {
        match value {
            KeyType::Delete => 0,
            KeyType::Put => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternalKey {
    pub user_key: Vec<u8>,
    pub sequence_number: u64,
    pub key_type: KeyType,
}

impl Eq for InternalKey {}

impl Ord for InternalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.user_key.cmp(&other.user_key) {
            Ordering::Equal => {}
            other_ordering => return other_ordering,
        }

        if self.sequence_number > other.sequence_number {
            return Ordering::Less;
        } else if self.sequence_number < other.sequence_number {
            return Ordering::Greater;
        }

        self.key_type.cmp(&other.key_type)
    }
}

impl PartialEq for InternalKey {
    fn eq(&self, other: &Self) -> bool {
        self.user_key == other.user_key
            && self.sequence_number == other.sequence_number
            && self.key_type == other.key_type
    }
}

impl PartialOrd for InternalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl InternalKey {
    pub fn new(user_key: Vec<u8>, sequence_number: u64, key_type: KeyType) -> Self {
        InternalKey {
            user_key,
            sequence_number,
            key_type,
        }
    }

    pub fn is_deletion(&self) -> bool {
        matches!(self.key_type, KeyType::Delete)
    }

    pub fn decode(data: &[u8]) -> Result<Self, StorageError> {
        let dl = data.len();
        if dl < 9 {
            return Err(StorageError::DecodeError(String::from(
                "expected minimum 9 bytes [8 sequence_number, 1 key_type]",
            )));
        }

        let seq: [u8; 8] = data[dl - 9..dl - 1].try_into().map_err(|_| {
            StorageError::DecodeError(String::from("could not decode sequence number"))
        })?;

        let seq_u64 = u64::from_be_bytes(seq);

        let kt = data.last().ok_or(StorageError::DecodeError(String::from(
            "could not decode last byte into key type",
        )))?;

        let key_type = KeyType::try_from(kt.to_owned())?;

        let user_key = data[0..dl - 9].to_vec();

        Ok(InternalKey {
            user_key,
            sequence_number: seq_u64,
            key_type,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::with_capacity(self.user_key.len() + 9);
        result.extend(&self.user_key);
        result.extend_from_slice(&self.sequence_number.to_be_bytes());
        result.push(u8::from(self.key_type));
        result
    }
}
