use crate::{vhd::VhdError, prelude::*};
use rdisk_shared::AsByteSliceMut;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct ParentLocatorRecord {
    platform_code: u32,
    platform_data_space: u32,
    platform_data_length: u32,
    reserved: u32,
    platform_data_offset: u64,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct VhdSparseHeaderRecord {
    cookie_id: u64,
    data_offset: u64,
    table_offset: u64,
    header_version: u32,
    max_table_entries: u32,
    block_size: u32,
    checksum: u32,
    parent_id: [u8; 16],
    parent_time_stamp: u32,
    reserved: u32,
    parent_unicode_name: [u16; 256],
    parent_locators: [ParentLocatorRecord; 8],
    padding: [u8; 256],
}

const SPARSE_COOKIE_ID: u64 = 0x6573_7261_7073_7863; // big endian "cxsparse"

impl VhdSparseHeaderRecord {
    fn swap_bytes(&mut self) {
        self.data_offset = self.data_offset.swap_bytes();
        self.table_offset = self.table_offset.swap_bytes();
        self.header_version = self.header_version.swap_bytes();
        self.max_table_entries = self.max_table_entries.swap_bytes();
        self.block_size = self.block_size.swap_bytes();
        self.checksum = self.checksum.swap_bytes();
        self.parent_time_stamp = self.parent_time_stamp.swap_bytes();

        for entry in &mut self.parent_locators {
            entry.platform_code = entry.platform_code.swap_bytes();
            entry.platform_data_space = entry.platform_data_space.swap_bytes();
            entry.platform_data_length = entry.platform_data_length.swap_bytes();
            entry.platform_data_offset = entry.platform_data_offset.swap_bytes();
        }
    }
}

pub struct SparseHeader {
    pub(crate) data_offset: u64,
    pub(crate) table_offset: u64,
    pub(crate) header_version: u32,
    pub(crate) max_table_entries: u32,
    pub(crate) block_size: u32,
    pub(crate) parent_id: Uuid,
    pub(crate) parent_time_stamp: u32,
    pub(crate) parent_name: String,
    pub(crate) parent_locators: [ParentLocatorRecord; 8],
}

impl SparseHeader {
    pub fn read(stream: &impl ReadAt, pos: u64) -> Result<Self> {

        let mut header = unsafe { rdisk_shared::StructBuffer::<VhdSparseHeaderRecord>::new() };
        stream.read_exact_at(pos, unsafe { header.as_byte_slice_mut() } )?;

        if SPARSE_COOKIE_ID != header.cookie_id {
            return Err(Error::from(VhdError::InvalidHeaderCookie));
        }

        header.swap_bytes();

        let checksum = calc_header_checksum!(header);
        if header.checksum != checksum {
            return Err(Error::from(VhdError::InvalidHeaderChecksum));
        }

        let parent_id = Uuid::from_bytes(header.parent_id);
        let safe_copy = header.parent_unicode_name; // parent_unicode_name is inside packed struct and requires unsafe block to borrow
        let parent_name = String::from_utf16_lossy(&safe_copy).trim_end_matches('\0').to_string();

        Ok(Self{
            data_offset: header.data_offset,
            table_offset: header.table_offset,
            header_version: header.header_version,
            max_table_entries: header.max_table_entries,
            block_size: header.block_size,
            parent_id,
            parent_time_stamp: header.parent_time_stamp,
            parent_name,
            parent_locators: header.parent_locators,
        })
    }
}