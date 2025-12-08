use fat32rs::*;

pub fn say_hello() {
    println!("Hello, world!");
}

#[test]
fn it_runs() {
    say_hello();
    assert_eq!(true, true);
}

fn main() {
    let img = std::fs::read("fat32.img").unwrap();
    let disk = fat32rs::RamDisk::new(&img, 512);
    let mut buf = [0u8; 512];
    disk.read(0, &mut buf).expect("read failed");
}
