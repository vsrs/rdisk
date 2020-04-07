use crate::prelude::*;

pub trait ReadAt {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize>;

    fn read_exact_at(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        let mut buffer = buffer;
        while !buffer.is_empty() {
            match self.read_at(offset, buffer) {
                Ok(0) => break, // EOF
                Ok(n) => buffer = &mut buffer[n..],
                Err(e) => return Err(e),
            }
        }

        if buffer.is_empty() {
            Ok(())
        } else {
            Err(Error::UnexpectedEOD)
        }
    }
}

pub trait WriteAt {
    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize>;

    fn write_all_at(&self, offset: u64, data: &[u8]) -> Result<()> {
        let mut offset = offset;
        let mut data = data;
        while !data.is_empty() {
            match self.write_at(offset, data) {
                Ok(0) => {
                    return Err(Error::WriteZero);
                }
                Ok(n) => {
                    data = &data[n..];
                    offset += n as u64;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

pub trait Disk: ReadAt + WriteAt {
    fn geometry(&self) -> Result<Geometry>;
    fn capacity(&self) -> Result<u64>;
    fn physical_sector_size(&self) -> Result<u32>;

    fn logical_sector_size(&self) -> Result<u32> {
        Ok(self.geometry()?.bytes_per_sector)
    }
}

pub trait DiskImage: Disk {
    const NAME: &'static str;
    const EXT: &'static [&'static str];

    /// returns the list of all virtual disk files
    fn backing_files(&self) -> Box<dyn core::iter::Iterator<Item = String>>;

    /// total size of all backing files
    fn storage_size(&self) -> Result<u64>;
}

pub trait ImageExtent: ReadAt + WriteAt {
    // fn backing_files(&self) -> Box<dyn Iterator<Item=&'_ str> + '_> {

    fn backing_files(&self) -> Box<dyn core::iter::Iterator<Item = String>>;
    fn storage_size(&self) -> Result<u64>;
}
