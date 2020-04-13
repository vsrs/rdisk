use rdisk::prelude::*;
use rdisk::vhd::{VhdImage, VhdKind};
use rdisk::mbr;

fn open_test_vhd(name: &str) -> Option<(VhdImage, String)> {
    std::env::var("CARGO_MANIFEST_DIR").ok().map(|dir|{
        let path = std::path::PathBuf::from(dir).join("testdata").join(name);
        let path = path.to_string_lossy().to_string();
        (VhdImage::open(path.clone()).unwrap(), path)
    })
}

fn check_no_read_footer(disk: &VhdImage) {
    let size = disk.capacity().unwrap();
    assert_eq!(3145728, size);

    let mut buffer = vec![0_u8; 2];
    let readed = disk.read_at(size-1, buffer.as_mut_slice()).unwrap();
    assert_eq!(1, readed);

    let readed = disk.read_at(size-2, buffer.as_mut_slice()).unwrap();
    assert_eq!(2, readed);

    let readed = disk.read_at(size, buffer.as_mut_slice()).unwrap();
    assert_eq!(0, readed);

    let read_err = disk.read_at(size+1, buffer.as_mut_slice()).unwrap_err();
    match read_err {
        Error::ReadBeyondEOD => (),
        _ => assert!(false)
    }
}

fn check_layout(disk: VhdImage) {
    let disk = PartitionedDisk::new(disk).unwrap();

    assert_eq!(1, disk.layout().partitions().count());
    assert_eq!(1, disk.partitions().count());

    let partition = disk.partitions().nth(0).unwrap();
    assert_eq!(65536, partition.offset());
    assert_eq!(2031616, partition.length());
    match partition.kind() {
        PartitionKind::Mbr(mbr::PartitionKind::Known(mbr::KnownPartitionKind::Fat16BLBA)) => (),
        _ => assert!(false)
    }
}

#[test]
fn fixed_vhd_read() {
    if let Some((disk, full_path)) = open_test_vhd("vhd_fixed_small.vhd") {
        check_no_read_footer(&disk);

        let mut files = disk.backing_files();
        assert_eq!(full_path, files.next().unwrap());
        assert_eq!(None, files.next()); // should be no more files

        check_layout(disk);
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

        check_layout(disk);
    }
}


#[test]
fn fixed_vhd_create() {
    let name = "sample.vhd";
    let size = 2*1024*1024;

    let _ = std::fs::remove_file(&name);

    let disk = VhdImage::create_fixed(name, size).unwrap();
    disk.write_at(size/2, b"asdf").unwrap();
    drop(disk);

    let disk = VhdImage::open(name).unwrap();
    assert_eq!(size, disk.capacity().unwrap());
    assert!(VhdKind::Fixed == disk.kind());
    let mut buffer = vec![0; 4];
    disk.read_at(size/2, &mut buffer).unwrap();
    assert_eq!(buffer, b"asdf");
    drop(disk);

    let _ = std::fs::remove_file(&name);
}