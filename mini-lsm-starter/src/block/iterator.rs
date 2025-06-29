#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use std::sync::Arc;

use crate::key::{Key, KeySlice, KeyVec};

use super::Block;

/// Iterates on a block.
pub struct BlockIterator {
    /// The internal `Block`, wrapped by an `Arc`
    block: Arc<Block>,
    /// The current key, empty represents the iterator is invalid
    key: KeyVec,
    /// the current value range in the block.data, corresponds to the current key
    value_range: (usize, usize),
    /// Current index of the key-value pair, should be in range of [0, num_of_elements)
    idx: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> Self {
        Self {
            block,
            key: KeyVec::new(),
            value_range: (0, 0),
            idx: 0,
            first_key: KeyVec::new(),
        }
    }

    /// Creates a block iterator and seek to the first entry.
    pub fn create_and_seek_to_first(block: Arc<Block>) -> Self {
        let mut result = Self::new(block);
        result.seek_to_first();
        result.first_key = result.key.clone();
        return result;
    }

    /// Creates a block iterator and seek to the first key that >= `key`.
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        let mut result = Self::new(block);
        result.seek_to_first();
        result.first_key = result.key.clone();
        result.seek_to_key(key);
        return result;
    }

    /// Returns the key of the current entry.
    pub fn key(&self) -> KeySlice {
        return KeySlice::from_slice(self.key.as_key_slice().raw_ref());
    }

    /// Returns the value of the current entry.
    pub fn value(&self) -> &[u8] {
        let data_slice = self.block.data.as_slice();
        return &data_slice[self.value_range.0..self.value_range.1];
    }

    /// Returns true if the iterator is valid.
    /// Note: You may want to make use of `key`
    pub fn is_valid(&self) -> bool {
        return !self.key.is_empty();
    }

    /// Seeks to the first key in the block.
    pub fn seek_to_first(&mut self) {
        self.seek_to_idx(0);
    }

    /// Move to the next key in the block.
    pub fn next(&mut self) {
        self.seek_to_idx(self.idx + 1);
    }

    /// Seek to the first key that >= `key`.
    /// Note: You should assume the key-value pairs in the block are sorted when being added by
    /// callers.
    pub fn seek_to_key(&mut self, key: KeySlice) {
        let lower_bound = self
            .block
            .offsets
            .binary_search_by(|offset| {
                let offset: usize = (*offset).try_into().unwrap();

                let key_len =
                    u16::from_be_bytes([self.block.data[offset], self.block.data[offset + 1]]);
                let key_len: usize = key_len.try_into().unwrap();

                let data_slice = self.block.data.as_slice();
                let current_key =
                    KeySlice::from_slice(&data_slice[offset + 2..offset + 2 + key_len]);
                match current_key.cmp(&key) {
                    std::cmp::Ordering::Equal => std::cmp::Ordering::Greater,
                    ord => ord,
                }
            })
            .unwrap_err();
        self.seek_to_idx(lower_bound);
    }

    fn seek_to_idx(&mut self, idx: usize) {
        if idx >= self.block.offsets.len() {
            self.key = Key::new();
            self.idx = 0;
            self.value_range.0 = 0;
            self.value_range.1 = 0;
            return;
        }

        let offset = self.block.offsets[idx];
        let offset: usize = offset.try_into().unwrap();

        let key_len = u16::from_be_bytes([self.block.data[offset], self.block.data[offset + 1]]);
        let key_len: usize = key_len.try_into().unwrap();

        let data_slice = self.block.data.as_slice();

        self.key.set_from_slice(Key::from_slice(
            &data_slice[offset + 2..offset + 2 + key_len],
        ));

        let value_len = u16::from_be_bytes([
            self.block.data[offset + 2 + key_len],
            self.block.data[offset + 2 + key_len + 1],
        ]);
        let value_len: usize = value_len.try_into().unwrap();

        self.value_range.0 = offset + 2 + key_len + 2;
        self.value_range.1 = offset + 2 + key_len + 2 + value_len;

        self.idx = idx;
    }
}
