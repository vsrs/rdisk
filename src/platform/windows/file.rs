use crate::{ReadAt, Result, WriteAt};
use nt_native::NtString;

type NtFile = nt_native::File;

#[derive(Debug)]
pub struct File(NtFile);

impl ReadAt for File {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        nt_native::ReadAt::read_at(&self.0, offset, buffer).map_err(From::from)
    }
}

impl WriteAt for File {
    fn write_at(&self, offset: u64, data: &[u8]) -> Result<usize> {
        nt_native::WriteAt::write_at(&self.0, offset, data).map_err(From::from)
    }
}

impl File {
    pub fn open(path: &str) -> Result<Self> {
        let nt_path = NtString::from(path);
        NtFile::open(&nt_path)
            .map(|nt_file| File(nt_file))
            .map_err(From::from)
    }

    pub fn create_preallocated(path: &str, size: u64) -> Result<Self> {
        let nt_path = NtString::from(path);
        NtFile::create_preallocated(&nt_path, size)
            .map(|nt_file| File(nt_file))
            .map_err(From::from)
    }

    pub fn size(&self) -> Result<u64> {
        self.0.size().map_err(From::from)
    }
}
