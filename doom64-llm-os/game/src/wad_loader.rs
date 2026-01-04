// WAD file loader

pub struct WAD {
    data: alloc::vec::Vec<u8>,
}

pub fn load_default() -> WAD {
    // Stub
    WAD {
        data: alloc::vec::Vec::new(),
    }
}

pub fn load_wad_from_disk(_path: &str) -> Result<WAD, &'static str> {
    // Stub
    Ok(WAD {
        data: alloc::vec::Vec::new(),
    })
}
