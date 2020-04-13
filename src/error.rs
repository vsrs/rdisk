use crate::xstd::String;

#[derive(Debug)]
pub enum Error {
    InvalidGptMbr,
    InvalidGptHeader,
    InvalidGptCrc,

    ReadBeyondEOD,
    WriteBeyondEOD,
    UnexpectedEOD, //
    WriteZero,
    NotFound(String),

    Platform(crate::platform::Error),
    Vhd(crate::vhd::VhdError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidGptMbr => write!(f, "Invalid GPT protective MBR"),
            Error::InvalidGptHeader => write!(f, "Invalid GPT header"),
            Error::InvalidGptCrc => write!(f, "Invalid GPT CRC"),
            Error::UnexpectedEOD => write!(f, "Unexpected end of data"),
            Error::ReadBeyondEOD => write!(f, "Read beyound end of data"),
            Error::WriteBeyondEOD => write!(f, "Write beyound end of data"),
            Error::WriteZero => write!(f, "Failed to write whole buffer"),
            Error::NotFound(ref s) => write!(f, "'{}' not found", s),
            Error::Platform(ref e) => e.fmt(f),
            Error::Vhd(ref e) => e.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Platform(ref e) => Some(e),
            _ => None,
        }
    }
}
