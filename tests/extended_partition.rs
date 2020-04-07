use rdisk::*;

// https://sourceforge.net/projects/dftt/

fn dump_layout(layout: &DiskLayout) {
    match layout {
        DiskLayout::Mbr(mbr) => {
            for p in mbr.partitions() {
                println!(
                    "@{}, end: {}, length: {}",
                    p.offset / 512,
                    p.offset / 512 + p.length / 512 - 1,
                    p.length / 512
                );
            }
            println!("---");
            for p in mbr.extended_partitions() {
                println!(
                    "@{}, end: {}, length: {}",
                    p.offset / 512,
                    p.offset / 512 + p.length / 512 - 1,
                    p.length / 512
                );
            }
        }
        _ => (),
    }
}

fn dump_image_info<T: DiskImage>(image: &T) {
    println!("{} image, size: {:?}", T::NAME, image.storage_size());
    for file in image.backing_files() {
        println!("  - {}", file);
    }
}

#[test]
fn extended_partition() {
    println!();

    if let Ok(d) = PhysicalDisk::open(1) {
        let disk = PartitionedDisk::new(d).unwrap();
        dump_layout(disk.layout());

        let detected_geometry = Geometry::detect(&*disk);
        println!("Reported geomentry: {}", disk.geometry().unwrap());
        println!("Detected geomentry: {:?}", detected_geometry);
    }

    if let Ok(path) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = std::path::PathBuf::from(path).join("testdata\\ext-part-test-2.dd");
        if path.exists() {
            let image = raw::RawDiskImage::open(&path.to_string_lossy().to_string()).unwrap();
            dump_image_info(&image);
    
            let layout = DiskLayout::read(&image).unwrap();
            dump_layout(&layout);
    
            for area in layout.partitions() {
                println!("@{} : {}", area.offset / 512, area.length / 512);
            }
        }
        else {
            print!("No test data, skipped ... ")
        }
    }

    /*
    @63, end: 52415, length: 52353
    @52416, end: 104831, length: 52416
    @104832, end: 157247, length: 52416
    @157311, end: 209663, length: 52353
    @209727, end: 262079, length: 52353
    @262143, end: 312479, length: 50337
    ---
    @157248, end: 312479, length: 155232
    @262080, end: 312479, length: 50400
        */
}
