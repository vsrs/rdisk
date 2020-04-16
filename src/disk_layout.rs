use crate::{gpt, mbr, tools, Disk, Result};

pub enum PartitionKind {
    Free,
    Mbr(mbr::PartitionKind),
    Gpt(uuid::Uuid),
}

impl From<&mbr::PartitionInfo> for PartitionKind {
    fn from(info: &mbr::PartitionInfo) -> Self {
        match info.kind {
            mbr::PartitionKind::Known(mbr::KnownPartitionKind::Empty) => PartitionKind::Free,
            _ => PartitionKind::Mbr(info.kind),
        }
    }
}

impl From<&gpt::PartitionInfo> for PartitionKind {
    fn from(info: &gpt::PartitionInfo) -> Self {
        if info.kind == uuid::Uuid::nil() {
            PartitionKind::Free
        } else {
            PartitionKind::Gpt(info.kind)
        }
    }
}

pub struct PartitionInfo {
    #[allow(clippy::large_enum_variant)] // for Raw
    pub offset: u64,
    pub length: u64,
    pub kind: PartitionKind,
}

impl From<&mbr::PartitionInfo> for PartitionInfo {
    fn from(info: &mbr::PartitionInfo) -> Self {
        PartitionInfo {
            offset: info.offset,
            length: info.length,
            kind: PartitionKind::from(info),
        }
    }
}

impl From<&gpt::PartitionInfo> for PartitionInfo {
    fn from(info: &gpt::PartitionInfo) -> Self {
        PartitionInfo {
            offset: info.offset,
            length: info.length,
            kind: PartitionKind::from(info),
        }
    }
}

impl PartitionInfo {
    pub fn raw(length: u64) -> Self {
        PartitionInfo {
            offset: 0,
            length,
            kind: PartitionKind::Free,
        }
    }
}
#[allow(clippy::large_enum_variant)] // for Raw
pub enum DiskLayout {
    Mbr(mbr::Layout),
    Gpt(gpt::Layout),
    Raw(u64), // entire disk length
}

impl DiskLayout {
    pub fn read(disk: &impl Disk) -> Result<DiskLayout> {
        let mbr: mbr::MasterBootRecord = tools::read_disk_struct(disk, 0)?;

        if !mbr.is_valid() {
            return Ok(DiskLayout::Raw(disk.capacity()?));
        }

        if mbr.is_gpt_protective() {
            let layout = gpt::Layout::read(disk, mbr)?;
            return Ok(DiskLayout::Gpt(layout));
        }

        let layout = mbr::Layout::read(disk, mbr)?;
        Ok(DiskLayout::Mbr(layout))
    }

    pub fn partitions(&self) -> DiskLayoutParts<'_> {
        DiskLayoutParts { layout: self, index: 0 }
    }
}

pub struct DiskLayoutParts<'d> {
    layout: &'d DiskLayout,
    index: usize,
}

impl core::iter::Iterator for DiskLayoutParts<'_> {
    type Item = PartitionInfo;

    fn next(&mut self) -> core::option::Option<Self::Item> {
        match self.layout {
            DiskLayout::Gpt(gpt) => gpt.partitions().get(self.index).map(|p| {
                self.index += 1;
                PartitionInfo::from(p)
            }),
            DiskLayout::Mbr(mbr) => mbr.partitions().get(self.index).map(|p| {
                self.index += 1;
                PartitionInfo::from(p)
            }),
            DiskLayout::Raw(disk_capacity) => match self.index {
                0 => {
                    self.index += 1;
                    Some(PartitionInfo::raw(*disk_capacity))
                }
                _ => None,
            },
        }
    }
}

impl core::iter::ExactSizeIterator for DiskLayoutParts<'_> {
    fn len(&self) -> usize {
        match self.layout {
            DiskLayout::Gpt(gpt) => gpt.partitions().len(),
            DiskLayout::Mbr(mbr) => mbr.partitions().len(),
            DiskLayout::Raw(_) => 1,
        }
    }
}
