use crate::vhd::FORMAT_VERSION;
use crate::{prelude::*, vhd::VhdError};
use rdisk_shared::{AsByteSliceMut, StructBuffer};

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
pub(crate) struct VhdSparseHeaderRecord {
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
    pub data_offset: u64,
    pub table_offset: u64,
    pub header_version: u32,
    pub max_table_entries: u32,
    pub block_size: u32,
    pub parent_id: Uuid,
    pub parent_name: String,
    pub(crate) _parent_time_stamp: u32,
    pub(crate) _parent_locators: [ParentLocatorRecord; 8],
}

impl SparseHeader {
    pub(crate) fn new(capacity: u64, table_offset: u64, block_size: u32) -> Self {
        // Looks like a typo in the docs:
        //     It is currently unused by existing formats and should be set to 0xFFFFFFFF,
        //
        // All windows generated VHDs contains 0xFFFFFFFFFFFFFFFF
        const DATA_OFFSET: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        Self {
            data_offset: DATA_OFFSET,
            table_offset,
            header_version: FORMAT_VERSION, // The only existing version
            max_table_entries: math::ceil(capacity, block_size as u64) as u32,
            block_size,
            parent_id: Uuid::nil(),
            parent_name: String::new(),
            _parent_time_stamp: 0,
            _parent_locators: unsafe { core::mem::zeroed() },
        }
    }

    pub(crate) fn read(stream: &impl ReadAt, pos: u64) -> Result<Self> {
        let mut header = unsafe { StructBuffer::<VhdSparseHeaderRecord>::new() };
        stream.read_exact_at(pos, unsafe { header.as_byte_slice_mut() })?;

        if SPARSE_COOKIE_ID != header.cookie_id {
            return Err(Error::from(VhdError::InvalidHeaderCookie));
        }

        header.swap_bytes();

        let checksum = calc_header_checksum!(header);
        if header.checksum != checksum {
            return Err(Error::from(VhdError::InvalidHeaderChecksum));
        }

        let parent_id = UuidEx::from_be_bytes(header.parent_id);
        let safe_copy = unsafe { &header.parent_unicode_name }; // parent_unicode_name is inside packed struct and requires unsafe block to borrow
        let parent_name = String::from_utf16_lossy(safe_copy).trim_end_matches('\0').to_string();

        Ok(Self {
            data_offset: header.data_offset,
            table_offset: header.table_offset,
            header_version: header.header_version,
            max_table_entries: header.max_table_entries,
            block_size: header.block_size,
            parent_id,
            _parent_time_stamp: header.parent_time_stamp,
            parent_name,
            _parent_locators: header.parent_locators,
        })
    }

    pub(crate) fn write(&self, stream: &impl WriteAt, pos: u64) -> Result<()> {
        let mut header = StructBuffer::<VhdSparseHeaderRecord>::zeroed();
        header.cookie_id = SPARSE_COOKIE_ID;
        header.data_offset = self.data_offset;
        header.table_offset = self.table_offset;
        header.header_version = self.header_version;
        header.max_table_entries = self.max_table_entries;
        header.block_size = self.block_size;

        let checksum = super::calc_header_bytes_checksum(&header);
        header.checksum = checksum;
        header.swap_bytes();

        stream.write_all_at(pos, header.buffer())
    }
}
