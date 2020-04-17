use super::*;
use crate::{math, sizes};

pub use sparse::SparseHeader;

pub struct VhdImage {
    footer: Footer,
    extent: Box<dyn VhdImageExtent>,
}

impl Drop for VhdImage{
    fn drop(&mut self) {
        let res = self.flush();
        debug_assert!( res.is_ok() );
    }
}

impl ReadAt for VhdImage {
    fn read_at(&self, offset: u64, data: &mut [u8]) -> Result<usize> {
        match math::bound_to(self.capacity()?, offset, data.len()) {
            Some(data_len) => self.extent.read_at(offset, &mut data[..data_len]),
            None => Err(Error::ReadBeyondEOD),
        }
    }
}

impl WriteAt for VhdImage {
    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        match math::bound_to(self.capacity()?, offset, data.len()) {
            Some(data_len) => self.extent.write_at(offset, &data[..data_len]),
            None => Err(Error::WriteBeyondEOD),
        }
    }
}

impl Flush for VhdImage {
    fn flush(&self) -> Result<()> { 
        self.extent.write_footer(&self.footer)?;
        self.extent.flush()
    }
}

impl Disk for VhdImage {
    fn geometry(&self) -> Result<Geometry> {
        Ok(self.footer.geometry)
    }

    fn capacity(&self) -> Result<u64> {
        Ok(self.footer.current_size)
    }

    fn physical_sector_size(&self) -> Result<u32> {
        Ok(sizes::SECTOR)
    }
}

impl DiskImage for VhdImage {
    const NAME: &'static str = "VHD";
    const EXT: &'static [&'static str] = &["vhd"];

    fn backing_files(&self) -> Box<dyn core::iter::Iterator<Item = String>> {
        self.extent.backing_files()
    }

    fn storage_size(&self) -> Result<u64> {
        self.extent.storage_size()
    }
}

const MAX_VHD_SIZE: u64 = 2040 * sizes::GIB;
fn check_max_size(size: u64) -> Result<()> {
    if size > MAX_VHD_SIZE {
        return Err(Error::from(VhdError::DiskSizeTooBig));
    }

    Ok(())
}

impl VhdImage {
    pub fn create_fixed<S: Into<String>>(path: S, size: u64) -> Result<Self> {
        check_max_size(size)?;

        let path = path.into();
        let file = File::create_preallocated(&path, size + sizes::SECTOR_U64)?;
        let footer = Footer::new(size, VhdKind::Fixed);
        let extent: Box<dyn VhdImageExtent> = Box::new(FixedExtent::new(file, path));
        extent.write_footer(&footer)?;

        Ok(Self { footer, extent })
    }

    pub fn create_dynamic<S: Into<String>>(_path: S, size: u64) -> Result<Self> {
        check_max_size(size)?;

        todo!()
    }

    pub fn create_differencing<S: Into<String>>(_path: S, _parent: S) -> Result<Self> {
        todo!()
    }

    pub fn open<S: Into<String>>(path: S) -> Result<Self> {
        let path = path.into();
        let file = File::open(&path)?;
        let capacity = file.size()?;

        if capacity < sizes::SECTOR_U64 {
            return Err(crate::Error::from(VhdError::FileTooSmall));
        }

        let footer_pos = capacity - sizes::SECTOR_U64;
        let footer = Footer::read(&file, footer_pos)?;
        // Note: Versions previous to Microsoft Virtual PC 2004 create disk images that have a 511-byte disk footer.
        // So the hard disk footer can exist in the last 511 or 512 bytes of the file that holds the hard disk image.
        // At the moment rdisk does not support files with 511-bytes footer.

        let extent: Box<dyn VhdImageExtent> = match footer.disk_type {
            VhdKind::Fixed => Box::new(FixedExtent::new(file, path)),
            VhdKind::Dynamic | VhdKind::Differencing => Box::new(SparseExtent::open(file, path, footer.data_offset)?),
        };

        Ok(Self { footer, extent })
    }
}

impl VhdImage {
    pub fn kind(&self) -> VhdKind {
        self.footer.disk_type
    }

    pub fn id(&self) -> &Uuid {
        &self.footer.unique_id
    }

    pub fn footer(&self) -> &Footer {
        &self.footer
    }

    pub fn sparse_header(&self) -> Option<&SparseHeader> {
        self.extent.sparse_header()
    }
}
