use fat32rs::*;

fn main() {
    // 1. Load disk image (std only)
    let img = std::fs::read("image/fat32.img")
        .expect("failed to read FAT32 image");

    // 2. Wrap it in a block device
    let disk = RamDisk::new(&img, 512);

    // 3. Mount the filesystem
    let mut fs = Fs::new(disk)
        .expect("failed to mount FAT32");

    println!("Mounted FAT32 filesystem");

    // 4. List root directory
    let mut entries = [DirEntry {
        name: [0; 11],
        attr: 0,
        first_cluster: 0,
        size: 0,
    }; 64];

    let count = fs.ls(&mut entries)
        .expect("failed to list root directory");

    println!("Root directory entries:");
    for entry in &entries[..count] {
        // Convert FAT name to string for display
        let name = core::str::from_utf8(&entry.name)
            .unwrap_or("???")
            .trim();
        if entry.is_directory() {
            println!("{}/", name);
        } else {
            println!("{}", name);
        }
    }

    // 5. Example: cd into a directory
    fs.cd("TEST").expect("cd failed");

    println!("Changed directory to TEST");

    // 6. List the new directory
    let mut entries2 = [DirEntry {
        name: [0; 11],
        attr: 0,
        first_cluster: 0,
        size: 0,
    }; 64];

    let count2 = fs.ls(&mut entries2)
        .expect("failed to list TEST directory");

    println!("TEST directory entries:");
    for entry in &entries2[..count2] {
        let name = core::str::from_utf8(&entry.name)
            .unwrap_or("???")
            .trim();
        if entry.is_directory() {
            println!("{}/", name);
        } else {
            println!("{}", name);
        }
    }
}

