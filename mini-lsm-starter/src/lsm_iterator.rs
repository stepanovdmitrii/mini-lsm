#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use anyhow::{bail, Result};

use crate::{
    iterators::{merge_iterator::MergeIterator, StorageIterator},
    mem_table::MemTableIterator,
};

/// Represents the internal type for an LSM iterator. This type will be changed across the tutorial for multiple times.
type LsmIteratorInner = MergeIterator<MemTableIterator>;

pub struct LsmIterator {
    inner: LsmIteratorInner,
}

impl LsmIterator {
    pub(crate) fn new(iter: LsmIteratorInner) -> Result<Self> {
        let mut result = LsmIterator { inner: iter };
        while result.inner.is_valid() && result.inner.value().is_empty() {
            result.inner.next()?;
        }
        Ok(result)
    }
}

impl StorageIterator for LsmIterator {
    type KeyType<'a> = &'a [u8];

    fn is_valid(&self) -> bool {
        self.inner.is_valid()
    }

    fn key(&self) -> &[u8] {
        self.inner.key().raw_ref()
    }

    fn value(&self) -> &[u8] {
        self.inner.value()
    }

    fn next(&mut self) -> Result<()> {
        self.inner.next()?;
        while self.inner.is_valid() && self.inner.value().is_empty() {
            self.inner.next()?;
        }
        Ok(())
    }
}

/// A wrapper around existing iterator, will prevent users from calling `next` when the iterator is
/// invalid. If an iterator is already invalid, `next` does not do anything. If `next` returns an error,
/// `is_valid` should return false, and `next` should always return an error.
pub struct FusedIterator<I: StorageIterator> {
    iter: I,
    has_errored: bool,
}

impl<I: StorageIterator> FusedIterator<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            has_errored: false,
        }
    }
}

impl<I: StorageIterator> StorageIterator for FusedIterator<I> {
    type KeyType<'a>
        = I::KeyType<'a>
    where
        Self: 'a;

    fn is_valid(&self) -> bool {
        return !self.has_errored && self.iter.is_valid();
    }

    fn key(&self) -> Self::KeyType<'_> {
        if self.iter.is_valid() && !self.has_errored {
            return self.iter.key();
        }
        panic!("invalid usage")
    }

    fn value(&self) -> &[u8] {
        if self.iter.is_valid() && !self.has_errored {
            return self.iter.value();
        }
        panic!("invalid usage")
    }

    fn next(&mut self) -> Result<()> {
        if self.has_errored {
            bail!("iterator has errored");
        }
        if self.iter.is_valid() {
            match self.iter.next() {
                Ok(_) => {}
                Err(e) => {
                    self.has_errored = true;
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}
