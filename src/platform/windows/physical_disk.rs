use crate::prelude::*;
use crate::StorageDeviceInfo;
use nt_native::NtString;

#[derive(Clone)]
#[cfg_attr(any(feature = "std", test), derive(Debug))]
pub struct PhysicalDisk(nt_native::Disk);

impl ReadAt for PhysicalDisk {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        nt_native::ReadAt::read_at(&self.0, offset, buffer).map_err(From::from)
    }
}

impl WriteAt for PhysicalDisk {
    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        nt_native::WriteAt::write_at(&self.0, offset, data).map_err(From::from)
    }
}

impl Flush for PhysicalDisk {
    fn flush(&self) -> Result<()> { 
        nt_native::Flush::flush(&self.0).map_err(From::from)
    }
}

impl Disk for PhysicalDisk {
    fn geometry(&self) -> crate::Result<Geometry> {
        self.0.geometry().map_err(From::from).map(|raw| Geometry {
            bytes_per_sector: raw.BytesPerSector,
            sectors_per_track: raw.SectorsPerTrack,
            heads_per_cylinder: raw.TracksPerCylinder,
            cylinders: unsafe { *raw.Cylinders.QuadPart() as u64 },
        })
    }
    fn capacity(&self) -> crate::Result<u64> {
        self.0.capacity().map_err(From::from)
    }
    fn physical_sector_size(&self) -> crate::Result<u32> {
        unimplemented!()
    }
}

impl PhysicalDisk {
    pub fn open(index: u32) -> Result<Self> {
        let name = format!("\\\\.\\PhysicalDrive{}", index);
        Self::open_by_name(name.as_str())
    }

    /// Platform specific name like `\\.\PhysicalDrive0`  
    pub fn open_by_name(name: &str) -> Result<Self> {
        let nt_name = NtString::from(name);
        nt_native::Disk::open(&nt_name).map_err(From::from).map(|d| Self(d))
    }

    pub fn is_readonly(&self) -> Result<bool> {
        self.0.is_readonly().map_err(From::from)
    }

    pub fn is_offline(&self) -> Result<bool> {
        self.0.is_offline().map_err(From::from)
    }

    pub fn is_removable(&self) -> Result<bool> {
        self.0.is_removable().map_err(From::from)
    }

    pub fn is_trim_enabled(&self) -> Result<bool> {
        self.0.is_trim_enabled().map_err(From::from)
    }
    pub fn has_seek_penalty(&self) -> Result<bool> {
        self.0.has_seek_penalty().map_err(From::from)
    }

    pub fn device_number(&self) -> Result<u32> {
        self.0.device_number().map_err(From::from)
    }

    pub fn device_info(&self) -> Result<StorageDeviceInfo> {
        todo!()
    }
}
