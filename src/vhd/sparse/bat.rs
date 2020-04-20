use super::VhdError;
use crate::prelude::*;
use rdisk_shared::AsByteSlice;

pub struct Bat {
    entries: Vec<u32>,
}

impl Bat {
    pub fn new(entries_count: u32) -> Self {
        Self{ entries: vec![0xFF_FF_FF_FF; entries_count as usize] }
    }

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

    pub fn write(&self, stream: &impl WriteAt, offset: u64) -> Result<usize> {
        let mut temp = self.entries.clone();
        for entry in &mut temp {
            *entry = entry.swap_bytes();
        }

        // The BAT is always extended to a sector boundary.
        let size = math::round_up(self.entries.len() * 4, crate::sizes::SECTOR as usize );
        let mut buffer = vec![0xFF_u8; size];
        let data = unsafe{ temp.as_byte_slice() };
        buffer[..data.len()].copy_from_slice(data);

        stream.write_all_at(offset, &buffer)?;
        Ok(buffer.len())
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
