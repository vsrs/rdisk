use super::{VhdError, VhdKind};
use crate::prelude::*;
use rdisk_shared::{AsByteSlice, AsByteSliceMut, StructBuffer};

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct VhdDiskGeometry {
    cylinders: u16,
    heads: u8,
    sectors_per_track: u8,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub(crate) struct VhdFooterRecord {
    cookie_id: u64,
    features: u32,
    format_version: u32,
    data_offset: u64,
    timestamp: u32,
    creator_app_id: u32,
    creator_version: u32,
    creator_os_id: u32,
    original_size: u64,
    current_size: u64,
    disk_geometry: VhdDiskGeometry,
    disk_type: u32,
    checksum: u32,
    unique_id: [u8; 16],
    saved_state: u8,
    padding: [u8; 427],
}

const COOKIE_ID: u64 = 0x7869_7463_656e_6f63; // big endian "conectix"

impl VhdFooterRecord {
    fn swap_bytes(&mut self) {
        self.features = self.features.swap_bytes();
        self.format_version = self.format_version.swap_bytes();
        self.data_offset = self.data_offset.swap_bytes();
        self.timestamp = self.timestamp.swap_bytes();
        self.creator_version = self.creator_version.swap_bytes();
        self.original_size = self.original_size.swap_bytes();
        self.current_size = self.current_size.swap_bytes();
        self.disk_geometry.cylinders = self.disk_geometry.cylinders.swap_bytes();
        self.disk_type = self.disk_type.swap_bytes();
        self.checksum = self.checksum.swap_bytes();
    }
}

pub struct Footer {
    pub features: u32,
    pub format_version: u32,
    pub data_offset: u64,
    pub timestamp: u32,
    pub creator_app_id: u32,
    pub creator_version: u32,
    pub creator_os_id: u32,
    pub original_size: u64,
    pub current_size: u64,
    pub geometry: Geometry,
    pub disk_type: VhdKind,
    pub unique_id: Uuid,
    pub saved_state: u8,
}

impl Footer {
    pub(crate) fn new(size: u64, kind: VhdKind) -> Self {
        const FEATURES: u32 = 2;
        const FORMAT_VERSION: u32 = 0x00010000;
        const CREATOR_APP: u32 = 0x6b_73_64_72; // "rdsk"
        const CREATOR_VERSION: u32 = 0x00010000;

        const WIN_OS_ID: u32 = 0x6b326957; // "Wi2k"
                                           // const MAC_OS_ID : u32 = 0x2063614d; // "Mac "

        let data_offset: u64 = match kind {
            VhdKind::Fixed => 0xFFFF_FFFF_FFFF_FFFF,
            _ => crate::sizes::SECTOR_U64,
        };

        let timestamp = 0; // TODO
        let unique_id = Uuid::new_v4(); // TODO: v4 uses rand and in turn std

        Footer {
            features: FEATURES,
            format_version: FORMAT_VERSION,
            data_offset,
            timestamp,
            creator_app_id: CREATOR_APP,
            creator_version: CREATOR_VERSION,
            creator_os_id: WIN_OS_ID,
            original_size: size,
            current_size: size,
            geometry: Geometry::with_vhd_capacity(size),
            disk_type: kind,
            unique_id,
            saved_state: 0,
        }
    }

    pub(crate) fn read(stream: &impl ReadAt, pos: u64) -> Result<Self> {
        let mut footer = unsafe { rdisk_shared::StructBuffer::<VhdFooterRecord>::new() };
        stream.read_exact_at(pos, unsafe { footer.as_byte_slice_mut() })?;

        if COOKIE_ID != footer.cookie_id {
            return Err(Error::from(VhdError::InvalidHeaderCookie));
        }

        footer.swap_bytes();

        let checksum = calc_header_checksum!(footer);
        if footer.checksum != checksum {
            return Err(Error::from(VhdError::InvalidHeaderChecksum));
        }
        if footer.features & 2 != 2 {
            todo!("log warning");
        }

        let geometry = Geometry::chs(
            footer.disk_geometry.cylinders as u64,
            footer.disk_geometry.heads as u32,
            footer.disk_geometry.sectors_per_track as u32,
        );

        let disk_type: VhdKind = match num_traits::FromPrimitive::from_u32(footer.disk_type) {
            Some(kind) => kind,
            None => return Err(Error::from(VhdError::UnknownVhdType(footer.disk_type))),
        };

        let unique_id = Uuid::from_bytes(footer.unique_id);

        Ok(Footer {
            features: footer.features,
            format_version: footer.format_version,
            data_offset: footer.data_offset,
            timestamp: footer.timestamp,
            creator_app_id: footer.creator_app_id,
            creator_version: footer.creator_version,
            creator_os_id: footer.creator_os_id,
            original_size: footer.original_size,
            current_size: footer.current_size,
            geometry,
            disk_type,
            unique_id,
            saved_state: footer.saved_state,
        })
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let geometry = self.geometry.clone(); // to_vhd_safe() or to_bios_safe();
        let disk_geometry = VhdDiskGeometry {
            cylinders: geometry.cylinders as u16,
            heads: geometry.heads_per_cylinder as u8,
            sectors_per_track: geometry.sectors_per_track as u8,
        };

        use num_traits::ToPrimitive;
        let disk_type = self.disk_type.to_u32().unwrap();

        let mut footer = unsafe { StructBuffer::<VhdFooterRecord>::new() };
        footer.cookie_id = COOKIE_ID;
        footer.features = self.features;
        footer.format_version = self.format_version;
        footer.data_offset = self.data_offset;
        footer.timestamp = self.timestamp;
        footer.creator_app_id = self.creator_app_id;
        footer.creator_version = self.creator_version;
        footer.creator_os_id = self.creator_os_id;
        footer.original_size = self.original_size;
        footer.current_size = self.current_size;
        footer.disk_geometry = disk_geometry;
        footer.disk_type = disk_type;
        footer.checksum = 0;
        footer.unique_id = *self.unique_id.as_bytes();
        footer.saved_state = self.saved_state;

        let checksum = super::calc_header_bytes_checksum(&footer);
        footer.checksum = checksum;
        footer.swap_bytes();

        let slice = unsafe { footer.as_byte_slice() };
        slice.to_vec()
    }
}
