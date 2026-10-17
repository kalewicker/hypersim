//! Novelty archive: FIFO-evicting set of type-bin signatures.

use std::collections::VecDeque;
use ahash::AHashSet;

pub struct NoveltyArchive {
    capacity: usize,
    signatures: AHashSet<u64>,
    eviction_queue: VecDeque<u64>,
    evictions: usize,
    new_this_window: usize,
}

impl NoveltyArchive {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            signatures: AHashSet::new(),
            eviction_queue: VecDeque::new(),
            evictions: 0,
            new_this_window: 0,
        }
    }

    pub fn begin_window(&mut self) {
        self.new_this_window = 0;
    }

    pub fn add(&mut self, signature: u64) -> bool {
        if self.signatures.contains(&signature) {
            return false;
        }
        if self.signatures.len() >= self.capacity {
            if let Some(old) = self.eviction_queue.pop_front() {
                self.signatures.remove(&old);
                self.evictions += 1;
            }
        }
        self.signatures.insert(signature);
        self.eviction_queue.push_back(signature);
        self.new_this_window += 1;
        true
    }

    pub fn size(&self) -> usize {
        self.signatures.len()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn evictions(&self) -> usize {
        self.evictions
    }
    pub fn new_this_window(&self) -> usize {
        self.new_this_window
    }
    pub fn contains(&self, signature: u64) -> bool {
        self.signatures.contains(&signature)
    }
}
