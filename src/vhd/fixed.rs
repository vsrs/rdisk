use super::*;
use crate::ImageExtent;

pub struct FixedExtent {
    file: File,
    file_path: String,
}

// read_at and write_at offset args should be valid as they checked in the VhdImage

macro_rules! debug_check {
    ($s:ident, $offset:ident, $data:ident) => {
        debug_assert!(($offset + $data.len() as u64) <= $s.file.size().unwrap() - crate::sizes::SECTOR_U64);
    };
}

impl ReadAt for FixedExtent {
    fn read_at(&self, offset: u64, data: &mut [u8]) -> Result<usize> {
        debug_check!(self, offset, data);

        self.file.read_at(offset, data)
    }
}

impl WriteAt for FixedExtent {
    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        debug_check!(self, offset, data);

        self.file.write_at(offset, data)
    }
}

impl Flush for FixedExtent {
    fn flush(&self) -> Result<()> {
        self.file.flush()
    }
}

impl ImageExtent for FixedExtent {
    fn backing_files(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(core::iter::once(self.file_path.clone()))
    }
    fn storage_size(&self) -> Result<u64> {
        self.file.size()
    }
}

impl ImageExtentOps for FixedExtent {}

impl VhdImageExtent for FixedExtent {
    fn write_footer(&self, footer: &Footer) -> Result<()> {
        let bytes = footer.to_bytes();
        let pos = self.file.size()? - crate::sizes::SECTOR_U64;

        self.file.write_all_at(pos, &bytes)
    }

    fn sparse_header(&self) -> Option<&SparseHeader> {
        None
    }
}

impl FixedExtent {
    pub(crate) fn new(file: File, file_path: String) -> Self {
        Self { file, file_path }
    }
}
