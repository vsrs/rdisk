use crate::prelude::*;
use crate::{crc, math, mbr::MasterBootRecord, tools};

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct RawUuid(u32, u16, u16, [u8; 8]);

impl From<&RawUuid> for Uuid {
    fn from(raw: &RawUuid) -> Self {
        Uuid::from_fields(raw.0, raw.1, raw.2, &raw.3).unwrap()
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct Header {
    signature: u64,
    revision: u32,
    header_size: u32,
    header_crc32: u32,
    reserved: u32,
    current_lba: u64,
    copy_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_id: RawUuid, // [u8; 16],
    partition_table_lba: u64,
    partition_count: u32,
    partition_entry_size: u32,
    partition_array_crc32: u32,
}

const SIGNATURE: u64 = 0x5452_4150_2049_4645_u64;
const HEADER_SIZE: u32 = 92;
const REVISION: u32 = 0x0001_0000;

impl Header {
    pub fn is_valid(&self) -> bool {
        SIGNATURE == self.signature && self.crc() == self.header_crc32
    }

    pub fn crc(&self) -> u32 {
        let mut copy = *self;
        copy.header_crc32 = 0;
        crc::struct_crc32(&copy)
    }
}

impl crate::AsByteSlice for Header {
    unsafe fn as_byte_slice(&self) -> &[u8] {
        core::slice::from_raw_parts(
            self as *const Self as *const u8,
            core::mem::size_of::<Self>(),
        )
    }
}

#[repr(C, packed)]
struct RawPartitionRecord {
    partition_type: RawUuid,
    partition_id: RawUuid,
    first_lba: u64,
    last_lba: u64, // inclusive
    flags: u64,
    name: [u16; 36],
}

pub struct PartitionInfo {
    pub id: Uuid,
    pub kind: Uuid,
    pub offset: u64,
    pub length: u64,
    pub flags: u64,
    pub name: String,
}

pub struct Layout {
    _protective_mbr: MasterBootRecord,
    disk_id: Uuid,
    partitions: Vec<PartitionInfo>,
}

fn read_partitions(disk: &impl Disk, header: &Header) -> Result<Vec<PartitionInfo>> {
    unsafe {
        let sector_size = disk.logical_sector_size()?;
        let buffer_size = math::round_up(
            header.partition_count * header.partition_entry_size,
            sector_size,
        );
        let mut buffer = crate::alloc_buffer(buffer_size as usize);
        disk.read_exact_at(
            sector_size as u64 * header.partition_table_lba,
            buffer.as_mut_slice(),
        )?;

        let crc = crc::crc32(&buffer);
        if crc != header.partition_array_crc32 {
            todo!("Invalid partition table crc error"); // InvalidGptCrc
        }

        let mut partitions = Vec::<PartitionInfo>::new();
        for chunk in buffer.chunks_exact(header.partition_entry_size as usize) {
            let raw = &*(chunk.as_ptr() as *const RawPartitionRecord);
            let id = Uuid::from(&raw.partition_id);
            if id == Uuid::nil() {
                break;
            }

            let offset = raw.first_lba * sector_size as u64;
            let length = (raw.last_lba - raw.first_lba + 1) * sector_size as u64;

            partitions.push(PartitionInfo {
                id,
                kind: Uuid::from(&raw.partition_type),
                offset,
                length,
                flags: raw.flags,
                name: String::from_utf16_lossy(&raw.name)
                    .trim_end_matches('\0')
                    .to_string(), // TODO: FromWide trait
            });
        }

        Ok(partitions)
    }
}

impl Layout {
    pub(crate) fn read(disk: &impl Disk, mbr: MasterBootRecord) -> Result<Layout> {
        if !mbr.is_gpt_protective() {
            todo!("Return invalid MBR error") // InvalidGptMbr
        }

        let sector_size = disk.logical_sector_size()? as u64;
        let mut header: Header = tools::read_disk_struct(disk, sector_size)?;
        let mut valid = header.is_valid();
        if !valid {
            // TODO: log warning

            // let's try second one
            let size = disk.capacity()?;
            let secondary_header: Header = tools::read_disk_struct(disk, size - sector_size)?;
            if secondary_header.is_valid() {
                header = secondary_header;
                valid = true;
            }
        }

        if !valid {
            todo!("Return invalid GPT header error") // InvalidGptHeader
        }

        if header.header_size != HEADER_SIZE {
            todo!("Log warning: unexpected header size")
        }
        if header.revision != REVISION {
            todo!("Log warning: unexpected revision")
        }

        let partitions = read_partitions(disk, &header)?;

        Ok(Layout {
            _protective_mbr: mbr,
            disk_id: Uuid::from(&header.disk_id),
            partitions,
        })
    }

    pub fn disk_id(&self) -> &uuid::Uuid {
        &self.disk_id
    }

    pub fn partitions(&self) -> &[PartitionInfo] {
        &self.partitions
    }
}
