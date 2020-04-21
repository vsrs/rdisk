mod shared;
use rdisk::{PartitionedDisk, PhysicalDisk};
use shared::*;

#[test]
fn hw_data() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[allow(non_snake_case)]
    struct PartitionInfo {
        DiskNumber: u32,
        PartitionNumber: u32,
        Offset: u64,
        Size: u64,
        AccessPaths: Option<Vec<String>>,
        Guid: Option<String>,
        GptType: Option<String>,
        Type: String,
        MbrType: Option<u32>,
        DriveLetter: Option<char>,
        IsBoot: bool,
        IsSystem: bool,
    }

    if let Some(dir) = get_testdata_path() {
        let json_file = dir.join("partitions.json");
        println!("Read from: {}", json_file.display());

        let data = std::fs::read_to_string(json_file).unwrap();
        let v: Vec<PartitionInfo> = serde_json::from_str(&data).unwrap();
        let mut disks = std::collections::HashMap::new();
        for item in v.into_iter() {
            disks
                .entry(item.DiskNumber)
                .or_insert_with(|| Vec::<PartitionInfo>::new())
                .push(item);
        }

        for (number, partitions) in disks {
            println!("Disk # {}", number);
            let disk = PhysicalDisk::open(number).unwrap();
            let disk = PartitionedDisk::new(disk).unwrap();

            for (hw, p) in partitions.iter().zip(disk.partitions()) {
                println!("    @{:13}, len: {:13}, kind: {:?}", hw.Offset, hw.Size, p.kind());
                assert_eq!(hw.Offset, p.offset());
                assert_eq!(hw.Size, p.length());
            }
        }
    }
}
