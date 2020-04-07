use crate::String;

#[derive(Debug, Copy, Clone, Hash)]
pub enum StorageBusType {
    Unknown,
    Ata,
    Scsi,
    Usb,
    Usb3,
    Iscsi,
    Sas,
    Sata,
    Nvme,
    Virtual,
}

impl core::fmt::Display for StorageBusType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StorageBusType {
    pub fn is_virtual(self) -> bool {
        match self {
            StorageBusType::Virtual => true,
            _ => false,
        }
    }

    /// USB devices may report its own storage device info, not the internal disk data.
    pub fn is_usb(self) -> bool {
        match self {
            StorageBusType::Usb | StorageBusType::Usb3 => true,
            _ => false,
        }
    }
}

pub struct StorageDeviceInfo {
    pub bus_type: StorageBusType,
    pub vendor_id: String,
    pub product_id: String,
    pub product_revision: String,
    pub serial_number: String,
}
