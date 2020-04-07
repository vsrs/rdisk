use crate::{ReadAt, WriteAt, StorageDeviceInfo, Disk, Result};

#[derive(Debug)]
pub struct PhysicalDisk();

impl ReadAt for PhysicalDisk {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        todo!()
    }
}

impl WriteAt for PhysicalDisk {
    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        todo!()
    }
}

impl Disk for PhysicalDisk {
    fn geometry(&self) -> crate::Result<&crate::Geometry> { unimplemented!() }
    fn capacity(&self) -> crate::Result<u64> { unimplemented!() }
    fn physical_sector_size(&self) -> crate::Result<u32> { unimplemented!() }
}

impl PhysicalDisk {
    pub fn open(index: u32) -> Result<Self> {
        let name = format!("\\\\.\\PhysicalDrive{}", index);
        Self::open_by_name(name.as_str())
    }

    /// Platform specific name like `\\.\PhysicalDrive0`  
    pub fn open_by_name(name: &str) -> Result<Self> {
        todo!()
    }

    pub fn is_readonly(&self) -> Result<bool> {
        todo!()
    }

    pub fn is_offline(&self) -> Result<bool> {
        todo!()
    }

    pub fn is_removable(&self) -> Result<bool> {
        todo!()
    }

    pub fn is_trim_enabled(&self) -> Result<bool> {
        todo!()
    }
    pub fn has_seek_penalty(&self) -> Result<bool> {
        todo!()
    }

    pub fn device_number(&self) -> Result<u32> {
        todo!()
    }

    pub fn device_info(&self) -> Result<StorageDeviceInfo> {
        todo!()
    }

    pub fn attributes(&self) -> Result<u64> {
        todo!()
    }
}
