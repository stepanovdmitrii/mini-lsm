#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::{BufMut, Bytes, BytesMut};
pub use iterator::BlockIterator;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the tutorial
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.data.len() + 2 * self.offsets.len() + 2);
        buf.put(self.data.as_slice());
        for offset in &self.offsets {
            buf.put_u16(*offset);
        }
        buf.put_u16(self.offsets.len().try_into().unwrap());
        return buf.freeze();
    }

    /// Decode from the data layout, transform the input `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        let mut count = u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]);
        let mut offsets: Vec<u16> = Vec::with_capacity(count.try_into().unwrap());
        let mut offset: usize = 0;
        while count > 0 {
            let key_len = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let key_len: usize = key_len.try_into().unwrap();
            let val_len =
                u16::from_be_bytes([data[offset + 2 + key_len], data[offset + 2 + key_len + 1]]);
            let val_len: usize = val_len.try_into().unwrap();
            offsets.push(offset.try_into().unwrap());
            offset += 2 + key_len + 2 + val_len;
            count -= 1;
        }
        let mut res_data: Vec<u8> = Vec::with_capacity(offset);
        res_data.extend_from_slice(&data[0..offset]);
        Block {
            data: res_data,
            offsets: offsets,
        }
    }
}
