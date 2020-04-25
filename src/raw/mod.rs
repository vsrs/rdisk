use crate::prelude::*;

#[derive(Clone)]
#[cfg_attr(any(feature = "std", test), derive(Debug))]
pub struct RawDiskImage {
    file: File, // TODO: replace with transport
    capacity: u64,
    geometry: Geometry,
    file_path: String,
}

impl ReadAt for RawDiskImage {
    fn read_at(&self, offset: u64, data: &mut [u8]) -> Result<usize> {
        self.file.read_at(offset, data)
    }
}

impl WriteAt for RawDiskImage {
    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        self.file.write_at(offset, data)
    }
}

impl Flush for RawDiskImage {
    fn flush(&self) -> Result<()> {
        self.file.flush().map_err(From::from)
    }
}

impl Disk for RawDiskImage {
    fn geometry(&self) -> Result<Geometry> {
        Ok(self.geometry)
    }

    fn capacity(&self) -> Result<u64> {
        Ok(self.capacity)
    }

    fn physical_sector_size(&self) -> Result<u32> {
        Ok(self.geometry.bytes_per_sector)
    }
}

impl DiskImage for RawDiskImage {
    const NAME: &'static str = "RAW";
    const EXT: &'static [&'static str] = &["dd", "img", "bin"];

    fn backing_files(&self) -> Box<dyn core::iter::Iterator<Item = String>> {
        Box::new(core::iter::once(self.file_path.clone()))
    }

    fn storage_size(&self) -> Result<u64> {
        Ok(self.capacity)
    }
}

impl RawDiskImage {
    pub fn open<S: Into<String>>(path: S) -> Result<Self> {
        let path = path.into();
        let file = File::open(&path)?;
        let capacity = file.size()?;

        let mut image = Self {
            file,
            capacity,
            geometry: Geometry::with_vhd_capacity(capacity),
            file_path: path,
        };

        // Make a better guess about the geometry based on MBR data
        if let Some(geometry) = Geometry::detect(&image)? {
            image.geometry = geometry;
        }

        if capacity % 512 != 0 {
            todo!("log warning")
        }

        if capacity < image.geometry.capacity() {
            todo!("log warning")
        }

        Ok(image)
    }
}
