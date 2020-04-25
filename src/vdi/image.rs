use crate::prelude::*;

pub struct VdiImage {
}

impl Drop for VdiImage {
    fn drop(&mut self) {
        let res = self.flush();
        debug_assert!(res.is_ok());
    }
}

impl ReadAt for VdiImage {
    fn read_at(&self, _offset: u64, _data: &mut [u8]) -> Result<usize> {
        todo!()
    }
}

impl WriteAt for VdiImage {
    fn write_at(&self, _offset: u64, _data: &[u8]) -> Result<usize> {
        todo!()
    }
}

impl Flush for VdiImage {
    fn flush(&self) -> Result<()> {
        todo!()
    }
}

impl Disk for VdiImage {
    fn geometry(&self) -> Result<Geometry> {
        todo!()
    }

    fn capacity(&self) -> Result<u64> {
        todo!()
    }

    fn physical_sector_size(&self) -> Result<u32> {
        todo!()
    }
}

impl DiskImage for VdiImage {
    const NAME: &'static str = "VDI";
    const EXT: &'static [&'static str] = &["vdi"];

    fn backing_files(&self) -> Box<dyn core::iter::Iterator<Item = String>> {
        todo!()
    }

    fn storage_size(&self) -> Result<u64> {
        todo!()
    }
}
