use crate::prelude::*;

pub struct Bat {
    entries: Vec<u32>,
}

impl Bat {
    pub fn read(stream: &impl ReadAt, offset: u64, entries_count: u32) -> Result<Self> {
        let entries_count = entries_count as usize;

        let mut table = Bat {
            entries: Vec::with_capacity(entries_count)
        };

        let buffer = unsafe {
            table.entries.set_len(entries_count);
            core::slice::from_raw_parts_mut(table.entries.as_mut_ptr() as *mut u8, entries_count * 4)
        };
        
        stream.read_exact_at(offset, buffer )?;

        for entry in &mut table.entries {
            *entry = entry.swap_bytes();
        }

        Ok(table)
    }
}

impl core::ops::Index<usize> for Bat {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }    
}