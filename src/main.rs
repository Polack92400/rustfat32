use fat32rs::*;

fn main() {
    // 1. Load disk image (std only)
    let img = std::fs::read("image/fat32.img")
        .expect("failed to read fat32 image");

    // 2. Wrap it in a block device
    let disk = RamDisk::new(&img, 512);

    // 3. Mount the filesystem
    let mut fs = Fs::new(disk)
        .expect("failed to mount FAT32");

    // 4. Demo navigation
    println!("Mounted FAT32 filesystem");

    // Example: cd into a directory
    fs.cd("TEST").expect("cd failed");

    println!("Changed directory to TEST");
}
