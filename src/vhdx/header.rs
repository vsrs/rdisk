use crate::prelude::*;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct FileIdRecord {
    signature: u64,
    creator: [u16; 512]
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct HeaderRecord {
    signature: u32,
    checksum: u32,
    sequence_number: u64,
    file_write_guid: Uuid,
    data_write_guid: Uuid,
    log_guid: Uuid,
    log_version: u16,
    version: u16,
    log_length: u32,
    log_offset: u64,
    reserved: [u8; 4016]
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct RegionTableHeader {
    signature: u32,
    checksum: u32,
    entry_count: u32,
    reserved: u32,
}

pub struct Header {

}