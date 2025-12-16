#![no_std]

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum DeviceError {
	OutOfBounds,
	ReadFailed,
}

pub trait DeviceBlock {
	fn read(&self, block_id: u64, buffer: &mut [u8]) -> Result<(), DeviceError>;
	fn block_size(&self) -> usize;
}

// RAM DISK
pub struct RamDisk<'a> {
	data: &'a [u8],
	block_size: usize,
}

impl<'a> RamDisk<'a> {
	pub fn new(data: &'a [u8], block_size: usize) -> Self {
		Self { data, block_size }
	}

	pub fn block_count(&self) -> usize {
		self.data.len() / self.block_size
	}
}

impl<'a> DeviceBlock for RamDisk<'a> {
	fn read(&self, block_id: u64, buffer: &mut [u8]) -> Result<(), DeviceError> {
		let block_id = block_id as usize;

		if buffer.len() != self.block_size {
			return Err(DeviceError::ReadFailed);
		}

		let start = block_id * self.block_size;
		let end = start + self.block_size;

		if end > self.data.len() {
			return Err(DeviceError::OutOfBounds);
		}

		buffer.copy_from_slice(&self.data[start..end]);
		Ok(())
	}

	fn block_size(&self) -> usize {
		self.block_size
	}
}

// BOOT SECTOR
#[derive(Debug, Clone, Copy)]
pub struct BootSector {
	pub bytes_per_sector: u16,
	pub sectors_per_cluster: u8,
	pub reserved_sectors: u16,
	pub fat_count: u8,
	pub fat_size_sectors: u32,
	pub root_cluster: u32,
}

impl BootSector {
	pub fn read_from<D: DeviceBlock>(device: &D) -> Result<Self, DeviceError> {
		let mut block = [0u8; 512];

		if device.block_size() != 512 {
			return Err(DeviceError::ReadFailed);
		}

		device.read(0, &mut block)?;

		let bytes_per_sector = u16::from_le_bytes([block[0x0B], block[0x0C]]);
		let sectors_per_cluster = block[0x0D];
		let reserved_sectors = u16::from_le_bytes([block[0x0E], block[0x0F]]);
		let fat_count = block[0x10];
		let fat_size_sectors =
			u32::from_le_bytes([block[0x24], block[0x25], block[0x26], block[0x27]]);
		let root_cluster =
			u32::from_le_bytes([block[0x2C], block[0x2D], block[0x2E], block[0x2F]]);

		Ok(Self {
			bytes_per_sector,
			sectors_per_cluster,
			reserved_sectors,
			fat_count,
			fat_size_sectors,
			root_cluster,
		})
	}
}

// FAT32 STRUCT
pub struct Fat32<D: DeviceBlock> {
	device: D,
	boot: BootSector,
	first_fat_sector: u64,
	first_data_sector: u64,
}

impl<D: DeviceBlock> Fat32<D> {
	pub fn new(device: D) -> Result<Self, DeviceError> {
		let boot = BootSector::read_from(&device)?;
		let first_fat_sector = boot.reserved_sectors as u64;
		let first_data_sector =
			first_fat_sector + (boot.fat_count as u64 * boot.fat_size_sectors as u64);

		Ok(Self {
			device,
			boot,
			first_fat_sector,
			first_data_sector,
		})
	}

	pub fn cluster_to_sector(&self, cluster: u32) -> u64 {
		let cluster_index = (cluster - 2) as u64;
		self.first_data_sector
			+ cluster_index * self.boot.sectors_per_cluster as u64
	}

	pub fn read_fat_entry(&self, cluster: u32) -> Result<u32, DeviceError> {
		let fat_offset = cluster as u64 * 4;
		let sector = self.first_fat_sector + fat_offset / self.boot.bytes_per_sector as u64;
		let offset = (fat_offset % self.boot.bytes_per_sector as u64) as usize;

		let mut buf = [0u8; 512];
		self.device.read(sector, &mut buf)?;

		let entry = u32::from_le_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
			& 0x0FFFFFFF;

		Ok(entry)
	}

	pub fn cluster_chain(&self, start: u32, out: &mut [u32]) -> Result<usize, DeviceError> {
		let mut current = start;
		let mut count = 0;

		loop {
			if count >= out.len() {
				break;
			}

			out[count] = current;
			count += 1;

			let next = self.read_fat_entry(current)?;
			if next >= 0x0FFFFFF8 {
				break;
			}

			current = next;
		}

		Ok(count)
	}

	pub fn read_directory_cluster(
		&self,
		cluster: u32,
		out: &mut [DirEntry],
	) -> Result<usize, DeviceError> {
		let first_sector = self.cluster_to_sector(cluster);
		let mut entry_count = 0;
		let mut sector_buf = [0u8; 512];

		for i in 0..self.boot.sectors_per_cluster {
			let sector = first_sector + i as u64;
			self.device.read(sector, &mut sector_buf)?;

			let mut offset = 0;
			while offset < 512 {
				if entry_count >= out.len() {
					return Ok(entry_count);
				}

				let entry_bytes = &sector_buf[offset..offset + 32];
				if let Some(entry) = DirEntry::from_bytes(entry_bytes) {
					out[entry_count] = entry;
					entry_count += 1;
				}

				offset += 32;
			}
		}

		Ok(entry_count)
	}
}

// DIRECTORY ENTRY
#[derive(Debug, Clone, Copy)]
pub struct DirEntry {
	pub name: [u8; 11],
	pub attr: u8,
	pub first_cluster: u32,
	pub size: u32,
}

impl DirEntry {
	pub fn is_directory(&self) -> bool {
		self.attr & 0x10 != 0
	}

	pub fn is_file(&self) -> bool {
		self.attr & 0x20 != 0
	}

	pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
		if bytes.len() != 32 {
			return None;
		}

		let first_byte = bytes[0];
		if first_byte == 0x00 || first_byte == 0xE5 {
			return None;
		}

		let attr = bytes[0x0B];
		if attr == 0x0F {
			return None;
		}

		let mut name = [0u8; 11];
		name.copy_from_slice(&bytes[0..11]);

		let cluster_high = u16::from_le_bytes([bytes[0x14], bytes[0x15]]) as u32;
		let cluster_low = u16::from_le_bytes([bytes[0x1A], bytes[0x1B]]) as u32;
		let first_cluster = (cluster_high << 16) | cluster_low;

		let size = u32::from_le_bytes([bytes[0x1C], bytes[0x1D], bytes[0x1E], bytes[0x1F]]);

		Some(Self {
			name,
			attr,
			first_cluster,
			size,
		})
	}

	pub fn matches_name(&self, name: &str) -> bool {
		let mut fat_name = [b' '; 11];
		let bytes = name.as_bytes();
		for i in 0..bytes.len().min(11) {
			fat_name[i] = bytes[i].to_ascii_uppercase();
		}
		self.name == fat_name
	}
}

// FILESYSTEM ABSTRACTION
pub struct Fs<D: DeviceBlock> {
	fat: Fat32<D>,
	cwd_cluster: u32,
}

impl<D: DeviceBlock> Fs<D> {
	pub fn new(device: D) -> Result<Self, DeviceError> {
		let fat = Fat32::new(device)?;
		let root = fat.boot.root_cluster;
		Ok(Self {
			fat,
			cwd_cluster: root,
		})
	}

	pub fn cd(&mut self, name: &str) -> Result<(), DeviceError> {
		let mut entries = [DirEntry {
			name: [0; 11],
			attr: 0,
			first_cluster: 0,
			size: 0,
		}; 64];

		let count = self.fat.read_directory_cluster(self.cwd_cluster, &mut entries)?;

		for entry in &entries[..count] {
			if entry.is_directory() && entry.matches_name(name) {
				self.cwd_cluster = entry.first_cluster;
				return Ok(());
			}
		}

		Err(DeviceError::ReadFailed)
	}
	pub fn ls<'a>(&self, out: &'a mut [DirEntry]) -> Result<usize, DeviceError> {
		self.fat.read_directory_cluster(self.cwd_cluster, out)
	}
}

