use rdisk::mbr;
use rdisk::prelude::*;
use rdisk::vhd::{VhdImage, VhdKind};
use rdisk_shared::AsByteSliceMut;

mod shared;
use shared::*;

fn check_no_read_footer(disk: &VhdImage) {
    let size = disk.capacity().unwrap();
    assert_eq!(3145728, size);

    let mut buffer = vec![0_u8; 2];
    let readed = disk.read_at(size - 1, buffer.as_mut_slice()).unwrap();
    assert_eq!(1, readed);

    let readed = disk.read_at(size - 2, buffer.as_mut_slice()).unwrap();
    assert_eq!(2, readed);

    let readed = disk.read_at(size, buffer.as_mut_slice()).unwrap();
    assert_eq!(0, readed);

    let read_err = disk.read_at(size + 1, buffer.as_mut_slice()).unwrap_err();
    match read_err {
        Error::ReadBeyondEOD => (),
        _ => assert!(false),
    }
}

fn check_mbr_layout(disk: VhdImage) {
    let disk = PartitionedDisk::new(disk).unwrap();

    assert_eq!(1, disk.layout().partitions().count());
    assert_eq!(1, disk.partitions().count());

    let partition = disk.partitions().nth(0).unwrap();
    assert_eq!(65536, partition.offset());
    assert_eq!(2031616, partition.length());
    match partition.kind() {
        PartitionKind::Mbr{ kind: mbr::PartitionKind::Known(mbr::KnownPartitionKind::Fat16BLBA), .. } => (),
        _ => assert!(false),
    }
}

#[test]
fn fixed_vhd_read() {
    if let Some((disk, full_path)) = open_test_vhd("vhd_fixed_small.vhd") {
        check_no_read_footer(&disk);

        let mut files = disk.backing_files();
        assert_eq!(full_path, files.next().unwrap());
        assert_eq!(None, files.next()); // should be no more files

        check_mbr_layout(disk);
    }
}

#[test]
fn dynamic_vhd_read() {
    if let Some((disk, full_path)) = open_test_vhd("vhd_dynamic_small.vhd") {
        let mut buffer: Vec<u8> = vec![0; 512];
        disk.read_at(510, buffer.as_mut_slice()).unwrap();
        assert_eq!(buffer[0], 0x55);
        assert_eq!(buffer[1], 0xAA);

        check_no_read_footer(&disk);

        let mut files = disk.backing_files();
        assert_eq!(full_path, files.next().unwrap());
        assert_eq!(None, files.next()); // should be no more files

        let id = format!("{:X}", disk.id());
        assert_eq!("ED51C3E2-93A7-4FF2-B7C4-D0B6407D49B0", id);

        check_mbr_layout(disk);
    }
}

#[test]
fn vhd_gpt_read() {
    if let Some((disk, _full_path)) = open_test_vhd("vhd_dynamic_small_gpt.vhd") {
        let disk = PartitionedDisk::new(disk).unwrap();

        assert_eq!(1, disk.layout().partitions().count());
        assert_eq!(1, disk.partitions().count());
    
        let partition = disk.partitions().nth(0).unwrap();
        assert_eq!(65536, partition.offset());
        assert_eq!(1048576, partition.length());
        match partition.kind() {
            PartitionKind::Gpt{ kind, id, name, flags} => {
                assert_eq!(kind, &Uuid::parse_str("ebd0a0a2-b9e5-4433-87c0-68b6b72699c7").unwrap());
                assert_eq!(id, &Uuid::parse_str("e0f1caa6-ff5a-434e-81c8-66baf88798bb").unwrap());
                assert_eq!(name, "Basic data partition");
                assert_eq!(flags, &0x8000_0000_0000_0000);
            },
            _ => assert!(false),
        }
    }
}

#[test]
fn dynamic_vhd_write() {
    if let Some((disk, full_path)) = open_test_vhd_copy("vhd_dynamic_small.vhd") {
        let mut buffer: Vec<u8> = vec![0; 512];
        disk.read_at(510, buffer.as_mut_slice()).unwrap();
        assert_eq!(buffer[0], 0x55);
        assert_eq!(buffer[1], 0xAA);

        let capacity = disk.capacity().unwrap();
        let template = 0xFF_00_AA_19_u32;
        // at the end of the disk
        disk.write_all_at(capacity - 4, &template.to_le_bytes()).unwrap();

        let details = disk.sparse_header().unwrap();

        // at the end of the first block
        disk.write_all_at(details.block_size as u64 - 2, &template.to_le_bytes()).unwrap();

        check_no_read_footer(&disk);

        let mut files = disk.backing_files();
        assert_eq!(full_path, files.next().unwrap());
        assert_eq!(None, files.next()); // should be no more files

        let mut temp: u32 = 0;
        disk.read_at(capacity - 4, unsafe { temp.as_byte_slice_mut() }).unwrap();
        assert_eq!(temp, template);

        disk.read_at(details.block_size as u64 - 2, unsafe { temp.as_byte_slice_mut() })
            .unwrap();
        assert_eq!(temp, template);

        check_mbr_layout(disk);
    }
}

#[test]
fn fixed_vhd_create() {
    let name = "sample.vhd";
    let size = 2 * 1024 * 1024;

    let _ = std::fs::remove_file(&name);

    let disk = VhdImage::create_fixed(name, size).unwrap();
    disk.write_at(size / 2, b"asdf").unwrap();
    drop(disk);

    let disk = VhdImage::open(name).unwrap();
    assert_eq!(size, disk.capacity().unwrap());
    assert!(VhdKind::Fixed == disk.kind());
    let mut buffer = vec![0; 4];
    disk.read_at(size / 2, &mut buffer).unwrap();
    assert_eq!(buffer, b"asdf");
    drop(disk);

    let _ = std::fs::remove_file(&name);
}

#[test]
fn dynamic_vhd_create() {
    let name = "sample_d.vhd";
    let size = 3 * 1024 * 1024;

    let _ = std::fs::remove_file(&name);

    let disk = VhdImage::create_dynamic(name, size).unwrap();
    drop(disk);

    let disk = VhdImage::open(name).unwrap();
    assert_eq!(size, disk.capacity().unwrap());
    assert!(VhdKind::Dynamic == disk.kind());

    disk.write_at(size / 2, b"asdf").unwrap();
    drop(disk);

    let disk = VhdImage::open(name).unwrap();
    let mut buffer = vec![0; 4];
    disk.read_at(size / 2, &mut buffer).unwrap();
    assert_eq!(buffer, b"asdf");
    drop(disk);

    let _ = std::fs::remove_file(&name);
}
