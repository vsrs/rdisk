use super::VhdError;
use crate::prelude::*;

pub struct Bat {
    entries: Vec<u32>,
}

impl Bat {
    pub fn read(stream: &impl ReadAt, offset: u64, entries_count: u32) -> Result<Self> {
        let entries_count = entries_count as usize;

        let mut table = Bat {
            entries: Vec::with_capacity(entries_count),
        };

        let buffer = unsafe {
            table.entries.set_len(entries_count);
            core::slice::from_raw_parts_mut(table.entries.as_mut_ptr() as *mut u8, entries_count * 4)
        };

        stream.read_exact_at(offset, buffer)?;

        for entry in &mut table.entries {
            *entry = entry.swap_bytes();
        }

        Ok(table)
    }

    pub fn block_id(&self, index: usize) -> Result<u32> {
        match self.entries.get(index) {
            Some(id) => Ok(*id),
            None => Err(Error::from(VhdError::InvalidBlockIndex(index))),
        }
    }

    /// The `index` MUST always be valid!
    pub fn set_block_id(&mut self, index: usize, id: u32) {
        self.entries[index] = id;
    }
}
