mod physical_disk;
pub use physical_disk::PhysicalDisk;

mod file;
pub use file::File;

pub type Error = nt_native::Error;

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        Self::Platform(err)
    }
}
