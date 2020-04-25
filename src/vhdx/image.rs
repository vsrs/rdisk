use crate::prelude::*;

pub struct VhdxImage {
}

impl Drop for VhdxImage {
    fn drop(&mut self) {
        let res = self.flush();
        debug_assert!(res.is_ok());
    }
}

impl ReadAt for VhdxImage {
    fn read_at(&self, _offset: u64, _data: &mut [u8]) -> Result<usize> {
        todo!()
    }
}

impl WriteAt for VhdxImage {
    fn write_at(&self, _offset: u64, _data: &[u8]) -> Result<usize> {
        todo!()
    }
}

impl Flush for VhdxImage {
    fn flush(&self) -> Result<()> {
        todo!()
    }
}

impl Disk for VhdxImage {
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

impl DiskImage for VhdxImage {
    const NAME: &'static str = "VHDX";
    const EXT: &'static [&'static str] = &["vhdx"];

    fn backing_files(&self) -> Box<dyn core::iter::Iterator<Item = String>> {
        todo!()
    }

    fn storage_size(&self) -> Result<u64> {
        todo!()
    }
}
