use core::f64;

use rand::Rng;

use crate::storage::internal_key::InternalKey;

#[derive(Debug)]
struct Node {
    key: Option<InternalKey>,
    value: Option<Vec<u8>>,
    forward_pointers: Vec<Option<Box<Node>>>,
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
    head_node: Box<Node>,
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
            head_node: Box::new(Node {
                key: None,
                value: None,
                forward_pointers,
            }),
            current_max_level: 0,
            length: 0,
        }
    }

    fn search(&mut self, key: &InternalKey) -> [*mut Node; MAX_HEIGHT] {
        let mut update: [*mut Node; MAX_HEIGHT] = [std::ptr::null_mut(); MAX_HEIGHT];

        let mut current: *mut Node = &mut *self.head_node as *mut Node;

        for level in (0..=self.current_max_level).rev() {
            // SAFETY: current is a valid pointer derived from self.head_node which is
            // owned by this SkipList and guaranteed to be valid for the lifetime of
            // this function. All nodes in forward_pointers are also valid as they're
            // owned by their predecessor nodes in the skiplist structure. We maintain
            // exclusive access through &mut self, preventing concurrent modification.
            unsafe {
                while let Some(ref mut next_box) = (&mut *current).forward_pointers[level] {
                    let next_key = next_box.key.as_ref().unwrap();

                    if next_key < key {
                        current = &mut **next_box as *mut Node;
                    } else {
                        break;
                    }
                }

                update[level] = current;
            }
        }

        update
    }

    fn random_height() -> usize {
        let mut rng = rand::rng();
        let mut height = 1;

        while rng.random::<f64>() < 0.5 && height < MAX_HEIGHT {
            height += 1;
        }

        height
    }
}
