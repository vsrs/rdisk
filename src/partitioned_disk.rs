use crate::*;

pub struct PartitionedDisk<D: Disk> {
    raw_disk: D,
    layout: DiskLayout,
}

impl<D: Disk> core::ops::Deref for PartitionedDisk<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.raw_disk
    }
}

impl<D: Disk> PartitionedDisk<D> {
    pub fn new(raw_disk: D) -> Result<Self> {
        let layout = DiskLayout::read(&raw_disk)?;
        Ok(Self { raw_disk, layout })
    }

    pub fn layout(&self) -> &DiskLayout {
        &self.layout
    }

    pub fn partitions(&self) -> Partitions<'_, D> {
        Partitions {
            disk: &self,
            iter: self.layout.partitions(),
        }
    }
}

pub struct Partitions<'d, D: Disk + 'd> {
    disk: &'d PartitionedDisk<D>,
    iter: DiskLayoutParts<'d>,
}

impl<'d, D: Disk + 'd> core::iter::Iterator for Partitions<'d, D> {
    type Item = Partition<'d, D>;

    fn next(&mut self) -> core::option::Option<Self::Item> {
        self.iter.next().map(|info| Partition::new(&self.disk.raw_disk, info))
    }
}

impl<'d, D: Disk + 'd> core::iter::ExactSizeIterator for Partitions<'d, D> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
