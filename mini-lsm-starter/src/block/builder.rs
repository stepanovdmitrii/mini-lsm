#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use nom::number::streaming::le_u16;

use crate::key::{KeySlice, KeyVec};

use super::Block;

/// Builds a block.
pub struct BlockBuilder {
    /// Offsets of each key-value entries.
    offsets: Vec<u16>,
    /// All serialized key-value pairs in the block.
    data: Vec<u8>,
    /// The expected block size.
    block_size: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockBuilder {
    /// Creates a new block builder.
    pub fn new(block_size: usize) -> Self {
        BlockBuilder {
            offsets: Vec::new(),
            data: Vec::new(),
            block_size: block_size,
            first_key: KeyVec::new(),
        }
    }

    /// Adds a key-value pair to the block. Returns false when the block is full.
    #[must_use]
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        if !self.is_empty() {
            let current_size = self.data.len() + 2 * self.offsets.len() + 1;
            let new_record_size = 4 + key.len() + value.len();
            if current_size + new_record_size > self.block_size {
                return false;
            }
        }

        if self.first_key.is_empty() {
            self.first_key = key.to_key_vec();
        }

        let key_length: u16 = key.len().try_into().unwrap();
        let value_length: u16 = value.len().try_into().unwrap();
        let offset: u16 = self.data.len().try_into().unwrap();

        self.data.append(&mut key_length.to_be_bytes().to_vec());
        self.data.extend_from_slice(key.raw_ref());

        self.data.append(&mut value_length.to_be_bytes().to_vec());
        self.data.extend_from_slice(value);

        self.offsets.push(offset);

        return true;
    }

    /// Check if there is no key-value pair in the block.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Finalize the block.
    pub fn build(self) -> Block {
        Block {
            data: self.data,
            offsets: self.offsets,
        }
    }
}
