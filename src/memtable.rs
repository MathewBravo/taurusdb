use crate::storage::internal_key::InternalKey;
use rand::Rng;
use std::cell::RefCell;
use std::io::Error;
use std::rc::Rc;
use std::thread::current;

type NodePtr = Rc<RefCell<Node>>;

#[derive(Debug)]
struct Node {
    key: Option<InternalKey>,
    value: Option<Vec<u8>>,
    forward_pointers: Vec<Option<NodePtr>>,
}

impl Node {
    fn new(key: InternalKey, value: Vec<u8>, height: usize) -> Self {
        let mut forward_pointers = Vec::with_capacity(height);
        for _ in 0..height {
            forward_pointers.push(None);
        }
        Node {
            key: Some(key),
            value: Some(value),
            forward_pointers,
        }
    }
}

const MAX_HEIGHT: usize = 12;

#[derive(Debug)]

pub struct SkipList {
    head_node: NodePtr,
    current_max_level: usize,
    length: usize,
}

impl SkipList {
    pub fn new() -> Self {
        let mut forward_pointers = Vec::with_capacity(MAX_HEIGHT);
        for _ in 0..MAX_HEIGHT {
            forward_pointers.push(None);
        }

        SkipList {
            head_node: Rc::new(RefCell::new(Node {
                key: None,
                value: None,
                forward_pointers,
            })),
            current_max_level: 0,
            length: 0,
        }
    }

    fn search(&self, key: &InternalKey) -> Vec<NodePtr> {
        let mut update: Vec<NodePtr> = Vec::with_capacity(MAX_HEIGHT);
        let mut current = Rc::clone(&self.head_node);

        for level in (0..=self.current_max_level).rev() {
            loop {
                let next_option = {
                    let current_node = current.borrow();
                    current_node.forward_pointers[level].clone()
                };

                match next_option {
                    Some(next) => {
                        let nn_ref = next.borrow();
                        let next_key = nn_ref.key.as_ref().unwrap();

                        if next_key < key {
                            current = next.clone();
                        } else {
                            break;
                        }
                    }
                    None => break,
                }
            }

            update.push(Rc::clone(&current));
        }

        update.reverse();
        update
    }

    pub fn insert(&mut self, key: InternalKey, value: Vec<u8>) -> Result<(), Error> {
        let update = self.search(&key);
        let current = update[0].clone();

        if let Some(next_node) = &current.borrow().forward_pointers[0] {
            let key_matches = {
                let nn_ref = next_node.borrow();

                nn_ref.key.as_ref() == Some(&key)
            };

            if key_matches {
                next_node.borrow_mut().value = Some(value);
                return Ok(());
            }
        }

        let height = Self::random_height();
        let new_node = Rc::new(RefCell::new(Node::new(key, value, height)));

        (0..height.min(self.current_max_level + 1)).for_each(|level| {
            new_node.borrow_mut().forward_pointers[level] =
                update[level].borrow().forward_pointers[level].clone();
            update[level].borrow_mut().forward_pointers[level] = Some(Rc::clone(&new_node));
        });

        if height > self.current_max_level + 1 {
            for level in (self.current_max_level + 1)..height {
                self.head_node.borrow_mut().forward_pointers[level] = Some(Rc::clone(&new_node));
            }
            self.current_max_level = height - 1;
        }

        self.length += 1;

        Ok(())
    }

    pub fn get(&self, key: &InternalKey) -> Option<Vec<u8>> {
        let update = self.search(key);
        let current = update[0].clone();

        if let Some(next_node) = &current.borrow().forward_pointers[0] {
            let nn = next_node.borrow();
            let next_key = nn.key.as_ref();
            if next_key == Some(key) {
                return nn.value.clone();
            }
        }

        None
    }

    pub fn delete(&mut self, key: &InternalKey) -> bool {
        let update = self.search(key);
        let current = update[0].clone();

        let node_to_delete = current.borrow().forward_pointers[0].clone();
        // node to delete
        if let Some(ntd) = &node_to_delete {
            let key_match = ntd.borrow().key.as_ref() == Some(key);

            if !key_match {
                return false;
            }

            let node_to_delete = current.borrow().forward_pointers[0].clone().unwrap();
            let node_height = node_to_delete.borrow().forward_pointers.len();

            (0..node_height.min(update.len())).for_each(|level| {
                update[level].borrow_mut().forward_pointers[level] =
                    node_to_delete.borrow().forward_pointers[level].clone();
            });

            while self.current_max_level > 0
                && self.head_node.borrow().forward_pointers[self.current_max_level].is_none()
            {
                self.current_max_level -= 1;
            }

            self.length -= 1;
            return true;
        }

        false
    }

    fn random_height() -> usize {
        let mut rng = rand::rng();

        let mut height = 1;
        while rng.random::<f64>() < 0.5 && height < MAX_HEIGHT {
            height += 1;
        }
        height
    }

    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

// While the code above was written by hand, I do not trust my knowledge of this system currently
// to hand write tests. These tests were generated by Gemini3 + GPT 5 after pasting the above code
// into GPT asking it to "Generate tests for my skiplist implementation without editing the
// existing code" and then giving those tests to Gemini3 and asking it to expand on them or alter
// them if necessary for better coverage.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::internal_key::{InternalKey, KeyType};

    // Helper to create test keys
    fn make_key(s: &str, seq: u64) -> InternalKey {
        InternalKey::new(s.as_bytes().to_vec(), seq, KeyType::Put)
    }

    #[test]
    fn test_new_skiplist_is_empty() {
        let sl = SkipList::new();

        assert_eq!(sl.len(), 0);
        assert!(sl.is_empty());
    }

    #[test]
    fn test_insert_and_get_single_item() {
        let mut sl = SkipList::new();
        let key = make_key("hello", 1);
        let value = b"world".to_vec();

        sl.insert(key.clone(), value.clone()).unwrap();

        assert_eq!(sl.len(), 1);
        assert_eq!(sl.get(&key), Some(value));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut sl = SkipList::new();
        let key1 = make_key("exists", 1);
        let key2 = make_key("missing", 2);

        sl.insert(key1, b"value".to_vec()).unwrap();

        assert_eq!(sl.get(&key2), None);
    }

    #[test]
    fn test_insert_multiple_items() {
        let mut sl = SkipList::new();

        sl.insert(make_key("apple", 1), b"red".to_vec()).unwrap();
        sl.insert(make_key("banana", 2), b"yellow".to_vec())
            .unwrap();
        sl.insert(make_key("cherry", 3), b"red".to_vec()).unwrap();

        assert_eq!(sl.len(), 3);
        assert_eq!(sl.get(&make_key("apple", 1)), Some(b"red".to_vec()));
        assert_eq!(sl.get(&make_key("banana", 2)), Some(b"yellow".to_vec()));
        assert_eq!(sl.get(&make_key("cherry", 3)), Some(b"red".to_vec()));
    }

    #[test]
    fn test_insert_updates_existing_key() {
        let mut sl = SkipList::new();
        let key = make_key("key", 1);

        sl.insert(key.clone(), b"value1".to_vec()).unwrap();
        assert_eq!(sl.len(), 1);
        assert_eq!(sl.get(&key), Some(b"value1".to_vec()));

        // Insert same key with new value

        sl.insert(key.clone(), b"value2".to_vec()).unwrap();
        assert_eq!(sl.len(), 1); // Length shouldn't increase
        assert_eq!(sl.get(&key), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_insert_in_sorted_order() {
        let mut sl = SkipList::new();

        // Insert in order

        sl.insert(make_key("a", 1), b"1".to_vec()).unwrap();
        sl.insert(make_key("b", 2), b"2".to_vec()).unwrap();
        sl.insert(make_key("c", 3), b"3".to_vec()).unwrap();

        assert_eq!(sl.get(&make_key("a", 1)), Some(b"1".to_vec()));
        assert_eq!(sl.get(&make_key("b", 2)), Some(b"2".to_vec()));
        assert_eq!(sl.get(&make_key("c", 3)), Some(b"3".to_vec()));
    }

    #[test]
    fn test_insert_in_reverse_order() {
        let mut sl = SkipList::new();

        // Insert in reverse order
        sl.insert(make_key("c", 3), b"3".to_vec()).unwrap();
        sl.insert(make_key("b", 2), b"2".to_vec()).unwrap();
        sl.insert(make_key("a", 1), b"1".to_vec()).unwrap();

        assert_eq!(sl.get(&make_key("a", 1)), Some(b"1".to_vec()));
        assert_eq!(sl.get(&make_key("b", 2)), Some(b"2".to_vec()));
        assert_eq!(sl.get(&make_key("c", 3)), Some(b"3".to_vec()));
    }

    #[test]
    fn test_insert_random_order() {
        let mut sl = SkipList::new();

        // Insert in random order

        sl.insert(make_key("m", 5), b"5".to_vec()).unwrap();
        sl.insert(make_key("a", 1), b"1".to_vec()).unwrap();
        sl.insert(make_key("z", 10), b"10".to_vec()).unwrap();
        sl.insert(make_key("d", 2), b"2".to_vec()).unwrap();
        sl.insert(make_key("p", 7), b"7".to_vec()).unwrap();

        assert_eq!(sl.len(), 5);
        assert_eq!(sl.get(&make_key("a", 1)), Some(b"1".to_vec()));
        assert_eq!(sl.get(&make_key("d", 2)), Some(b"2".to_vec()));
        assert_eq!(sl.get(&make_key("m", 5)), Some(b"5".to_vec()));
        assert_eq!(sl.get(&make_key("p", 7)), Some(b"7".to_vec()));
        assert_eq!(sl.get(&make_key("z", 10)), Some(b"10".to_vec()));
    }

    #[test]
    fn test_delete_existing_key() {
        let mut sl = SkipList::new();

        let key = make_key("key", 1);

        sl.insert(key.clone(), b"value".to_vec()).unwrap();
        assert_eq!(sl.len(), 1);

        let deleted = sl.delete(&key);
        assert!(deleted);

        assert_eq!(sl.len(), 0);
        assert_eq!(sl.get(&key), None);
    }

    #[test]
    fn test_delete_nonexistent_key() {
        let mut sl = SkipList::new();
        let key1 = make_key("exists", 1);
        let key2 = make_key("missing", 2);

        sl.insert(key1, b"value".to_vec()).unwrap();

        let deleted = sl.delete(&key2);
        assert!(!deleted);
        assert_eq!(sl.len(), 1);
    }

    #[test]
    fn test_delete_from_middle() {
        let mut sl = SkipList::new();

        sl.insert(make_key("a", 1), b"1".to_vec()).unwrap();
        sl.insert(make_key("b", 2), b"2".to_vec()).unwrap();
        sl.insert(make_key("c", 3), b"3".to_vec()).unwrap();

        let deleted = sl.delete(&make_key("b", 2));
        assert!(deleted);
        assert_eq!(sl.len(), 2);

        assert_eq!(sl.get(&make_key("a", 1)), Some(b"1".to_vec()));
        assert_eq!(sl.get(&make_key("b", 2)), None);
        assert_eq!(sl.get(&make_key("c", 3)), Some(b"3".to_vec()));
    }

    #[test]
    fn test_delete_first_item() {
        let mut sl = SkipList::new();

        sl.insert(make_key("a", 1), b"1".to_vec()).unwrap();

        sl.insert(make_key("b", 2), b"2".to_vec()).unwrap();
        sl.insert(make_key("c", 3), b"3".to_vec()).unwrap();

        let deleted = sl.delete(&make_key("a", 1));
        assert!(deleted);
        assert_eq!(sl.len(), 2);

        assert_eq!(sl.get(&make_key("a", 1)), None);
        assert_eq!(sl.get(&make_key("b", 2)), Some(b"2".to_vec()));
        assert_eq!(sl.get(&make_key("c", 3)), Some(b"3".to_vec()));
    }

    #[test]
    fn test_delete_last_item() {
        let mut sl = SkipList::new();

        sl.insert(make_key("a", 1), b"1".to_vec()).unwrap();

        sl.insert(make_key("b", 2), b"2".to_vec()).unwrap();
        sl.insert(make_key("c", 3), b"3".to_vec()).unwrap();

        let deleted = sl.delete(&make_key("c", 3));
        assert!(deleted);
        assert_eq!(sl.len(), 2);

        assert_eq!(sl.get(&make_key("a", 1)), Some(b"1".to_vec()));
        assert_eq!(sl.get(&make_key("b", 2)), Some(b"2".to_vec()));
        assert_eq!(sl.get(&make_key("c", 3)), None);
    }

    #[test]
    fn test_delete_all_items() {
        let mut sl = SkipList::new();

        sl.insert(make_key("a", 1), b"1".to_vec()).unwrap();

        sl.insert(make_key("b", 2), b"2".to_vec()).unwrap();
        sl.insert(make_key("c", 3), b"3".to_vec()).unwrap();

        assert!(sl.delete(&make_key("b", 2)));
        assert!(sl.delete(&make_key("a", 1)));
        assert!(sl.delete(&make_key("c", 3)));

        assert_eq!(sl.len(), 0);
        assert!(sl.is_empty());
    }

    #[test]

    fn test_large_dataset() {
        let mut sl = SkipList::new();
        let n = 1000;

        // Insert many items
        for i in 0..n {
            let key = make_key(&format!("key{:04}", i), i);
            let value = format!("value{}", i).into_bytes();
            sl.insert(key, value).unwrap();
        }

        assert_eq!(sl.len(), n as usize);

        // Verify all items exist
        for i in 0..n {
            let key = make_key(&format!("key{:04}", i), i);
            let expected = format!("value{}", i).into_bytes();
            assert_eq!(sl.get(&key), Some(expected));
        }

        // Delete half the items
        for i in (0..n).step_by(2) {
            let key = make_key(&format!("key{:04}", i), i);
            assert!(sl.delete(&key));
        }

        assert_eq!(sl.len(), (n / 2) as usize);

        // Verify deleted items are gone and remaining items exist
        for i in 0..n {
            let key = make_key(&format!("key{:04}", i), i);
            if i % 2 == 0 {
                assert_eq!(sl.get(&key), None);
            } else {
                let expected = format!("value{}", i).into_bytes();
                assert_eq!(sl.get(&key), Some(expected));
            }
        }
    }

    #[test]
    fn test_empty_value() {
        let mut sl = SkipList::new();
        let key = make_key("empty", 1);
        let value = Vec::new();

        sl.insert(key.clone(), value.clone()).unwrap();
        assert_eq!(sl.get(&key), Some(value));
    }

    #[test]
    fn test_large_value() {
        let mut sl = SkipList::new();
        let key = make_key("large", 1);
        let value = vec![42u8; 10000]; // 10KB value

        sl.insert(key.clone(), value.clone()).unwrap();
        assert_eq!(sl.get(&key), Some(value));
    }

    #[test]
    fn test_same_user_key_different_sequence() {
        let mut sl = SkipList::new();

        // InternalKey sorts by user_key first, then sequence descending
        let key1 = make_key("user", 10);
        let key2 = make_key("user", 5);
        let key3 = make_key("user", 1);

        sl.insert(key1.clone(), b"v10".to_vec()).unwrap();
        sl.insert(key2.clone(), b"v5".to_vec()).unwrap();
        sl.insert(key3.clone(), b"v1".to_vec()).unwrap();

        // All three should coexist (different InternalKeys)
        assert_eq!(sl.len(), 3);
        assert_eq!(sl.get(&key1), Some(b"v10".to_vec()));
        assert_eq!(sl.get(&key2), Some(b"v5".to_vec()));

        assert_eq!(sl.get(&key3), Some(b"v1".to_vec()));
    }
}
