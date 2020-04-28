#[derive(Copy, Clone, FromPrimitive, ToPrimitive, Eq, PartialEq)]
pub enum VhdxKind {
    Fixed,
    Dynamic,
    Differencing,
}

mod image;
pub use image::VhdxImage;

mod header;
pub use header::Header;