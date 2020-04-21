use rdisk::vhd::VhdImage;
use std::path::PathBuf;

pub fn get_testdata_path() -> Option<PathBuf> {
    std::env::var("CARGO_MANIFEST_DIR").ok().and_then(|dir| {
        let dir = PathBuf::from(dir).join("testdata");
        if dir.exists() {
            Some(dir)
        } else {
            None
        }
    })
}

pub fn open_test_vhd(name: &str) -> Option<(VhdImage, String)> {
    get_testdata_path().and_then(|dir| {
        let path = dir.join(name);
        let path = path.to_string_lossy().to_string();
        if let Ok(vhd) = VhdImage::open(path.clone()) {
            Some((vhd, path))
        } else {
            println!("No '{}'. Skipped.", path);
            None
        }
    })
}

pub fn open_test_vhd_copy(name: &str) -> Option<(VhdImage, String)> {
    get_testdata_path().and_then(|dir| {
        let from = dir.join(name);
        let copy_name = "copy_".to_string() + name;
        let _ = std::fs::remove_file(&copy_name);
        let to = dir.join(&copy_name);
        std::fs::copy(from, to).ok().and_then(|_| open_test_vhd(&copy_name))
    })
}
