#[derive(Debug)]
pub enum VhdError {
    FileTooSmall,
    InvalidHeaderCookie,
    InvalidHeaderChecksum,
    InvalidSparseHeaderCookie,
    InvalidSparseHeaderChecksum,
    InvalidSparseHeaderOffset,
    DiskSizeTooBig,
    UnknownVhdType(u32),
    InvalidBlockIndex(usize),
    UnexpectedBlockId(usize, u32), // the value returend from Bat::block_id()
}

impl core::fmt::Display for VhdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VhdError::FileTooSmall => f.write_str("File too small"),
            VhdError::InvalidHeaderCookie => f.write_str("Invalid VHD header cookie"),
            VhdError::InvalidHeaderChecksum => f.write_str("Invalid VHD header checksum"),
            VhdError::InvalidSparseHeaderCookie => f.write_str("Invalid VHD Sparse header cookie"),
            VhdError::InvalidSparseHeaderChecksum => f.write_str("Invalid VHD Sparse header checksum"),
            VhdError::InvalidSparseHeaderOffset => f.write_str("Invalid VHD Sparse header BAT offset"),
            VhdError::DiskSizeTooBig => f.write_str("Disk size too big for VHD"),
            VhdError::UnknownVhdType(n) => write!(f, "Unknown VHD type '{}'", n),
            VhdError::InvalidBlockIndex(idx) => write!(f, "Invalid block index '{}'", idx),
            VhdError::UnexpectedBlockId(idx, id) => write!(f, "Unexpected '{}' block id '{:08X}'", idx, id),
        }
    }
}

impl From<VhdError> for crate::Error {
    fn from(e: VhdError) -> Self {
        Self::Vhd(e)
    }
}
