
// ============================================================================
// mmap_bcm_register - convenience function used to map the GPIO register block
// ============================================================================
use crate::{BCM2709_PERI_BASE,REGISTER_BLOCK_SIZE};
use mmap::{MemoryMap, MapOption};
use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
use std::os::unix::io::AsRawFd;

pub fn mmap_bcm_register(register_offset: usize) -> Option<MemoryMap> {

    let mem_file =
        match OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open("/dev/mem") {
            Err(why) => panic!("couldn't open /dev/mem: {}", why),
            Ok(file) => file
        };

    let mmap_options = &[
        MapOption::MapNonStandardFlags(libc::MAP_SHARED),
        MapOption::MapReadable,
        MapOption::MapWritable,
        MapOption::MapFd(mem_file.as_raw_fd()),
        MapOption::MapOffset(BCM2709_PERI_BASE as usize + register_offset as usize)
    ];

    let result = MemoryMap::new(REGISTER_BLOCK_SIZE as usize, mmap_options).unwrap();

    return match result.data().is_null() {
        true => {
            eprintln!("mmap error: {}", std::io::Error::last_os_error());
            eprintln!("Pi3: MMapping from base 0x{:X}, offset 0x{:X}", BCM2709_PERI_BASE, register_offset);
            None
        },
        false => Some(result)
    };

    // NOTE/WARNING: When a MemoryMap struct is dropped, the mapped 
    // memory region is automatically unmapped!
}