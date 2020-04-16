use crate::prelude::*;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, FromPrimitive, ToPrimitive)]
pub enum KnownPartitionKind {
    Empty = 0x00,

    // FAT12 as primary partition in first physical 32 MB of disk or as logical drive anywhere on disk (else use 06h instead)
    Fat12 = 0x01,

    // FAT16 with less than 65536 sectors (32 MB).
    // As primary partition it must reside in first physical 32 MB of disk, or as logical drive anywhere on disk (else use 06h instead)
    Fat16 = 0x04,

    // Extended partition with CHS addressing. It must reside in first physical 8 GB of disk, else use 0Fh instead.
    ExtendedCHS = 0x05,

    // FAT16B with 65536 or more sectors.
    // It must reside in first physical 8 GB of disk, unless used for logical drives in an 0Fh extended partition (else use 0Eh instead).
    // Also used for FAT12 and FAT16 volumes in primary partitions if they are not residing in first physical 32 MB of disk
    Fat16BCHS = 0x06,

    // May be exFAT!!!
    Ntfs = 0x07,

    // FAT32 with CHS addressing
    Fat32CHS = 0x0B,

    // FAT32 with LBA addressing
    Fat32LBA = 0x0C,

    // FAT16B with LBA addressing
    Fat16BLBA = 0x0E,

    // Extended partition with LBA
    ExtendedLBA = 0x0F,

    // May be Fat32 or NTFS
    WindowsRecovery = 0x27,

    // Something related to MS Dynamic disks
    DynamicExtendedPartition = 0x42,

    // GPT
    GptProtectiveMBR = 0xEE,

    // very close to FAT, but see docs (UEFI 2_5.pdf)
    EfiSystemPartition = 0xEF,

    VmwareVmfs = 0xFB,
}

#[derive(Clone, Copy)]
pub enum PartitionKind {
    Known(KnownPartitionKind),
    Unknown(u8),
}

impl core::fmt::Display for PartitionKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PartitionKind::Known(k) => write!(f, "{:?}", k),
            PartitionKind::Unknown(id) => write!(f, "Unknown(0x{:X})", id),
        }
    }
}

impl From<u8> for PartitionKind {
    fn from(id: u8) -> Self {
        use num_traits::FromPrimitive;

        match KnownPartitionKind::from_u8(id) {
            Some(known_id) => PartitionKind::Known(known_id),
            None => PartitionKind::Unknown(id),
        }
    }
}

impl PartitionKind {
    pub fn is_extended(self) -> bool {
        match self {
            PartitionKind::Known(KnownPartitionKind::ExtendedLBA) => true,
            PartitionKind::Known(KnownPartitionKind::ExtendedCHS) => true,
            _ => false,
        }
    }
}

#[repr(C, packed)]
pub(crate) struct PartitionRecord {
    pub(crate) bootstrap_flags: u8,
    pub(crate) starting_head: u8,
    pub(crate) starting_sector: u8,
    pub(crate) starting_cylinder: u8,
    pub(crate) partition_kind: u8,
    pub(crate) end_head: u8,
    pub(crate) end_sector: u8,
    pub(crate) end_cylinder: u8,
    pub(crate) first_sector_lba: u32,
    pub(crate) partition_size_in_sectors: u32, // contains 0xffffffff if partition size in sectors does not fit u32
}

#[repr(C, packed)]
pub(crate) struct MasterBootRecord {
    pub(crate) boot_code: [u8; 440],
    pub(crate) disk_signature: u32,
    pub(crate) is_copy_protected: u16, // zeroes or 0x5A5A if copyprotected
    pub(crate) partition_table: [PartitionRecord; 4],
    pub(crate) signature: u16,
}

const SIGNATURE: u16 = 0xAA55;

impl MasterBootRecord {
    pub fn is_valid(&self) -> bool {
        SIGNATURE == self.signature
    }

    pub fn is_gpt_protective(&self) -> bool {
        self.is_valid() && self.partition_table[0].partition_kind == KnownPartitionKind::GptProtectiveMBR as u8
    }
}

pub struct PartitionInfo {
    pub offset: u64,
    pub length: u64,
    pub kind: PartitionKind,
    pub boot_indicator: bool,
}

impl PartitionInfo {
    fn new(record: &PartitionRecord, sector_size: u64, relative_offset: u64) -> Self {
        let offset = record.first_sector_lba as u64 * sector_size + relative_offset;
        let length = record.partition_size_in_sectors as u64 * sector_size;
        let boot = (record.bootstrap_flags & 0x80) == 0x80;

        Self {
            offset,
            length,
            kind: PartitionKind::from(record.partition_kind),
            boot_indicator: boot,
        }
    }
}

pub struct Layout {
    mbr: MasterBootRecord,
    extended_partitions: Vec<PartitionInfo>,
    partitions: Vec<PartitionInfo>,
}

fn read_extended_partition(
    disk: &impl Disk,
    offset: u64,
    partitions: &mut Vec<PartitionInfo>,
    extended_partitions: &mut Vec<PartitionInfo>,
) -> Result<()> {
    let sector_size = disk.logical_sector_size()? as u64;
    let mut ebr_offset = offset;

    'main: loop {
        let ebr: MasterBootRecord = tools::read_disk_struct(disk, ebr_offset)?;
        if !ebr.is_valid() {
            // TODO: log warning;
            break;
        }

        let record = &ebr.partition_table[0];
        if record.first_sector_lba != 0 {
            let info = PartitionInfo::new(record, sector_size, ebr_offset);
            partitions.push(info);
        }

        for next_record in &ebr.partition_table[1..] {
            if next_record.first_sector_lba == 0 {
                break 'main;
            }

            let info = PartitionInfo::new(next_record, sector_size, ebr_offset);
            if info.kind.is_extended() {
                extended_partitions.push(info);
                ebr_offset = offset + next_record.first_sector_lba as u64 * sector_size;
                break;
            } else {
                partitions.push(info);
            }
        }
    }

    Ok(())
}

impl Layout {
    pub(crate) fn read(disk: &impl Disk, mbr: MasterBootRecord) -> Result<Layout> {
        let sector_size = disk.logical_sector_size()? as u64;
        let mut partitions = Vec::<PartitionInfo>::new();
        let mut extended_partitions = Vec::<PartitionInfo>::new();

        for entry in &mbr.partition_table {
            if entry.first_sector_lba == 0 || entry.partition_size_in_sectors == 0 {
                continue;
            }

            let info = PartitionInfo::new(&entry, sector_size, 0);
            if info.kind.is_extended() {
                let offset = info.offset;
                extended_partitions.push(info);
                read_extended_partition(disk, offset, &mut partitions, &mut extended_partitions)?;
            } else {
                partitions.push(info);
            }
        }

        Ok(Layout {
            mbr,
            extended_partitions,
            partitions,
        })
    }

    pub fn disk_signature(&self) -> u32 {
        self.mbr.disk_signature
    }

    pub fn partitions(&self) -> &[PartitionInfo] {
        &self.partitions
    }

    pub fn has_extended_partition(&self) -> bool {
        !self.extended_partitions.is_empty()
    }

    pub fn extended_partitions(&self) -> &[PartitionInfo] {
        &self.extended_partitions
    }
}
