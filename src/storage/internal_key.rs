use core::cmp::Ordering;

#[derive(Debug, PartialEq, PartialOrd)]
enum KeyType {
    Delete,
    Put,
}

#[derive(Debug)]
pub struct InternalKey {
    user_key: Vec<u8>,
    sequence_number: u64,
    key_type: KeyType,
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

        if self.key_type == KeyType::Delete && other.key_type == KeyType::Put {
            return Ordering::Less;
        } else if self.key_type == KeyType::Put && other.key_type == KeyType::Delete {
            return Ordering::Greater;
        }

        Ordering::Equal
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
    pub fn decode(data: &[u8]) -> Self {}

    pub fn encode(&self) -> Vec<u8> {}
}
