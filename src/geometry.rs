use crate::{math, mbr, sizes, tools, Disk};

#[derive(Debug, Copy, Clone)]
pub struct Geometry {
    pub cylinders: u64,
    pub heads_per_cylinder: u32,
    pub sectors_per_track: u32,
    pub bytes_per_sector: u32, // logical sector size, for 512e may be less then physical sector size
}

impl core::fmt::Display for Geometry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.bytes_per_sector == sizes::SECTOR {
            write!(
                f,
                "({}/{}/{})",
                &self.cylinders, &self.heads_per_cylinder, &self.sectors_per_track
            )
        } else {
            write!(
                f,
                "({}/{}/{}:{})",
                &self.cylinders,
                &self.heads_per_cylinder,
                &self.sectors_per_track,
                &self.bytes_per_sector
            )
        }
    }
}

impl Geometry {
    pub fn chs(cylinders: u64, heads_per_cylinder: u32, sectors_per_track: u32) -> Self {
        Self {
            cylinders,
            heads_per_cylinder,
            sectors_per_track,
            bytes_per_sector: sizes::SECTOR,
        }
    }

    /// https://en.wikipedia.org/wiki/Logical_block_addressing#LBA-assisted_translation
    pub fn lba_assisted(capacity: u64) -> Self {
        let heads: u32 = match capacity {
            c if c <= 504 * sizes::MIB => 16,
            c if c <= 1008 * sizes::MIB => 32,
            c if c <= 2016 * sizes::MIB => 64,
            c if c <= 4032 * sizes::MIB => 128,
            _ => 255,
        };

        let sectors = 63_u32;
        let cylinders = core::cmp::min(
            1024_u64,
            capacity / (sectors * heads * sizes::SECTOR) as u64,
        );
        Geometry::chs(cylinders, heads, sectors)
    }

    /// calculates a geometry using the VHD algorithm and default sector size (512 bytes).
    pub fn with_vhd_capacity(capacity: u64) -> Self {
        Self::with_vhd_capacity_and_sector(capacity, sizes::SECTOR)
    }

    /// calculates a geometry using the VHD algorithm and custom sector size.
    pub fn with_vhd_capacity_and_sector(capacity: u64, sector_size: u32) -> Self {
        //                                Cylinders   Heads    Sectors
        let total_sectors = if capacity > 65535_u64 * 16_u64 * 255_u64 * sector_size as u64 {
            65535_u32 * 16_u32 * 255_u32
        } else {
            capacity as u32 / sector_size
        };

        let (heads_per_cylinder, sectors_per_track) = if total_sectors > 65535_u32 * 16_u32 * 63_u32
        {
            (255, 16)
        } else {
            let mut sectors_per_track = 17_u32;
            let mut cylinders_times_heads = total_sectors / sectors_per_track;
            let mut heads_per_cylinder = (cylinders_times_heads + 1023) / 1024;

            if heads_per_cylinder < 4 {
                heads_per_cylinder = 4
            }

            if cylinders_times_heads >= heads_per_cylinder * 1024 || heads_per_cylinder > 16 {
                sectors_per_track = 31;
                heads_per_cylinder = 16;
                cylinders_times_heads = total_sectors / sectors_per_track;
            }

            if cylinders_times_heads >= heads_per_cylinder * 1024 {
                sectors_per_track = 63;
                heads_per_cylinder = 16;
            }

            (heads_per_cylinder, sectors_per_track)
        };

        let cylinders = total_sectors / sectors_per_track / heads_per_cylinder;

        Geometry {
            cylinders: cylinders as u64,
            heads_per_cylinder,
            sectors_per_track,
            bytes_per_sector: sector_size,
        }
    }

    /// Tries to get the disk geometry from MBR
    pub fn detect(disk: &impl Disk) -> crate::Result<Option<Self>> {
        tools::read_disk_struct(disk, 0)
            .and_then(|header: mbr::MasterBootRecord| {
                if header.is_valid() {
                    let mut max_head = 0_u32;
                    let mut max_sector = 0_u32;
                    for p in &header.partition_table {
                        max_head = core::cmp::max(max_head, p.end_head as u32);
                        max_sector = core::cmp::max(max_sector, p.end_sector as u32);
                    }

                    if max_head != 0 && max_sector != 0 {
                        max_head += 1;
                        let sector_size = disk.logical_sector_size()?;
                        let cylinder_size = max_head * max_sector * sector_size;
                        let cylinders = math::ceil(disk.capacity()?, cylinder_size as u64);

                        return Ok(Some(Geometry {
                            cylinders: cylinders as u64,
                            heads_per_cylinder: max_head,
                            sectors_per_track: max_sector,
                            bytes_per_sector: sector_size,
                        }));
                    }
                }

                Ok(None)
            })
    }
}

impl Geometry {
    pub fn capacity(&self) -> u64 {
        self.capacity_in_sectors() * (self.bytes_per_sector as u64)
    }

    pub fn capacity_in_sectors(&self) -> u64 {
        self.cylinders * (self.heads_per_cylinder as u64) * (self.sectors_per_track as u64)
    }
}
