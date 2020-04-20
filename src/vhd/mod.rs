use crate::prelude::*;
use rdisk_shared::AsByteSlice;

pub(crate) fn calc_header_bytes_checksum<H: AsByteSlice>(header: &H) -> u32 {
    let mut new_checksum = 0_u32;
    for b in unsafe { header.as_byte_slice() } {
        new_checksum += *b as u32;
    }

    !new_checksum
}

macro_rules! calc_header_checksum {
    ($header:ident) => {{
        let mut copy = $header.clone();
        copy.checksum = 0;

        crate::vhd::calc_header_bytes_checksum(&copy)
    }};
}

const FEATURES: u32 = 2;
const FORMAT_VERSION: u32 = 0x0001_0000;
pub const CREATOR_APP: u32 = 0x6b_73_64_72; // "rdsk"
pub const CREATOR_VERSION: u32 = 0x0001_0000;
pub const WIN_OS_ID: u32 = 0x6b_32_69_57; // "Wi2k"
pub const MAC_OS_ID: u32 = 0x20_63_61_4d; // "Mac "

mod footer;
use footer::*;

mod error;
pub use error::VhdError;

mod image;
pub use image::*;

mod fixed;
pub use fixed::*;

mod sparse;
pub use sparse::*;

trait VhdImageExtent: ImageExtent + ImageExtentOps {
    fn write_footer(&self, footer: &Footer) -> Result<()>;
    fn sparse_header(&self) -> Option<&SparseHeader>;
}

#[derive(Copy, Clone, FromPrimitive, ToPrimitive, Eq, PartialEq)]
pub enum VhdKind {
    Fixed = 2,
    Dynamic = 3,
    Differencing = 4,
}

// TODO: test https://github.com/wdormann/vhds
