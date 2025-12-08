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
