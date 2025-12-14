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
