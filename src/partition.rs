use crate::prelude::*;
use crate::{PartitionInfo, PartitionKind};

pub struct Partition<'d, D: Disk + 'd> {
    disk: &'d D,
    info: PartitionInfo,
}

impl<'d, D: Disk + 'd> ReadAt for Partition<'d, D> {
    fn read_at(&self, offset: u64, data: &mut [u8]) -> Result<usize> {
        self.disk.read_at(self.info.offset + offset, data)
    }
}

impl<'d, D: Disk + 'd> Partition<'d, D> {
    pub(crate) fn new(disk: &'d D, info: PartitionInfo) -> Self {
        Self { disk, info }
    }

    #[inline]
    pub fn offset(&self) -> u64 {
        self.info.offset
    }

    #[inline]
    pub fn length(&self) -> u64 {
        self.info.length
    }

    #[inline]
    pub fn kind(&self) -> &PartitionKind {
        &self.info.kind
    }
}
