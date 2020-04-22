#![cfg_attr(not(feature = "std"), no_std)]

pub(crate) use rdisk_shared::*;

#[macro_use]
extern crate num_derive;

mod error;
pub use error::*;
pub type Result<T> = core::result::Result<T, Error>;

// reexport
pub use uuid::Uuid;

pub mod sizes {
    pub const SECTOR: u32 = 512;
    pub const SECTOR_U64: u64 = SECTOR as u64;
    pub const KIB: u64 = 1024;
    pub const MIB: u64 = 1024 * KIB;
    pub const GIB: u64 = 1024 * MIB;
}

pub mod crc;
pub mod gpt;
pub mod math;
pub mod mbr;

pub mod raw;
pub mod sdi;
pub mod vdi;
pub mod vhd;
pub mod vhdx;
pub mod vmdk;
pub mod wim;

mod device_info;
pub use device_info::*;

mod geometry;
pub use geometry::Geometry;

mod traits;
pub use traits::*;

mod disk_layout;
pub use disk_layout::*;

mod partition;
pub use partition::*;

mod partitioned_disk;
pub use partitioned_disk::*;

pub(crate) mod platform;
pub use platform::{File, PhysicalDisk};

pub mod prelude {
    pub use crate::Uuid;
    pub(crate) use crate::{crc, math, tools, ImageExtentOps};
    pub use crate::{Disk, DiskImage, Error, File, Flush, Geometry, ImageExtent, ReadAt, Result, WriteAt};
    pub use crate::{Partition, PartitionInfo, PartitionKind, PartitionedDisk};
    pub(crate) use rdisk_shared::xstd::*;
}

pub(crate) mod tools {
    pub use super::*;

    pub fn read_disk_struct<T, D>(disk: &D, offset: u64) -> Result<T>
    where
        T: Sized,
        D: Disk,
    {
        debug_assert_eq!(core::mem::align_of::<T>(), 1);

        unsafe {
            let sector_size = disk.logical_sector_size()? as usize;
            debug_assert!(sector_size >= core::mem::size_of::<T>());

            let mut buffer = alloc_buffer(sector_size);
            disk.read_exact_at(offset, buffer.as_mut_slice())?;

            Ok(core::ptr::read(buffer.as_ptr() as *const T))
        }
    }
}
