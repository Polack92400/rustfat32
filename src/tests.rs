use super::*; 

#[cfg(test)] 
mod tests { 
	use super::*; 
	// Helper: create a simple disk with predictable bytes. 
	fn make_disk() -> RamDisk<'static> {
	// Disk contains bytes: 0,1,2,3,4,5,6,7...
	static DATA: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7,
	8, 9, 10,11,12,13,14,15];
	RamDisk::new(&DATA, 4) // 4-byte blocks → 4 blocks total
}

#[test]
fn test_block_size() {
	let disk = make_disk();
	assert_eq!(disk.block_size(), 4);
}

#[test]
fn test_block_count() {
	let disk = make_disk();
	assert_eq!(disk.block_count(), 16 / 4); // 4 blocks
}

#[test]
fn test_read_first_block() {
	let disk = make_disk();
	let mut buf = [0u8; 4];

	disk.read(0, &mut buf).unwrap();

	assert_eq!(buf, [0, 1, 2, 3]);
}

#[test]
fn test_read_second_block() {
	let disk = make_disk();
	let mut buf = [0u8; 4];

	disk.read(1, &mut buf).unwrap();

	assert_eq!(buf, [4, 5, 6, 7]);
}

#[test]
fn test_read_last_block() {
	let disk = make_disk();
	let mut buf = [0u8; 4];

	disk.read(3, &mut buf).unwrap();

	assert_eq!(buf, [12, 13, 14, 15]);
}

#[test]
fn test_out_of_bounds_block() {
	let disk = make_disk();
	let mut buf = [0u8; 4];

	let result = disk.read(4, &mut buf); // block 4 does not exist

	assert!(matches!(result, Err(DeviceError::OutOfBounds)));
}

#[test]
fn test_wrong_buffer_size() {
	let disk = make_disk();
	let mut buf = [0u8; 2]; // too small

	let result = disk.read(0, &mut buf);

	assert!(matches!(result, Err(DeviceError::ReadFailed)));
}
}
