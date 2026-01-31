use embedded_storage::{self, ReadStorage, Storage};
extern crate alloc;
use alloc::vec::Vec;

pub struct MemoryStorage {
    internal_memory: Vec<u8>,
}

#[derive(Debug)]
pub struct MemoryError;

impl MemoryStorage {
    pub fn new(size: usize) -> Self {
        MemoryStorage {
            internal_memory: alloc::vec![0; size],
        }
    }

    pub fn dump(&mut self) -> &[u8] {
        self.internal_memory.as_slice()
    }
}

impl ReadStorage for MemoryStorage {
    type Error = MemoryError;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let start = offset as usize;
        let end = (offset + bytes.len() as u32) as usize;

        if end > self.internal_memory.len() {
            return Err(MemoryError);
        }

        bytes.copy_from_slice(&self.internal_memory[start..end]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        return self.internal_memory.len();
    }
}

impl Storage for MemoryStorage {
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let start = offset as usize;
        let end = (offset + bytes.len() as u32) as usize;

        if end > self.internal_memory.len() {
            return Err(MemoryError);
        }

        self.internal_memory[start..end].copy_from_slice(bytes);
        Ok(())
    }
}

pub struct WeirdoFileSystem<T>
where
    T: Storage,
{
    pub storage: T,
    offset: u32,
    size: u32,
    empty_block: u32,
    total_blocks: u32,
}

#[derive(Debug)]
pub enum WeirdoFileSystemError {
    PayloadTooLarge,
    KeyNotFound,
}

const BLOCK_SIZE: u16 = 2048;
const BLOCK_META_DATA: u8 = 6;

impl<T> WeirdoFileSystem<T>
where
    T: Storage,
{
    pub fn new(storage: T, offset: u32, size:u32) -> Self {
        let mut new_fs = WeirdoFileSystem {
            storage,
            offset,
            size,
            empty_block: 0,
            total_blocks: 0,
        };
        new_fs.build_cache();
        new_fs
    }

    fn build_cache(&mut self) {
        self.empty_block = 0;
        self.total_blocks = (self.size as u32 / BLOCK_SIZE as u32);
        for block in 0..self.total_blocks {
            let address = self.offset + (block * BLOCK_SIZE as u32);
            let mut occupied = [0u8; 1];
            let _ = self.storage.read(address, &mut occupied);
            if occupied == [0x00] {
                return;
            } else {
                self.empty_block = self.empty_block + 1;
            }
        }
    }

    pub fn amount_of_free_blocks(&mut self) -> u32 {
        self.total_blocks - self.empty_block
    }

    pub fn write_key_value(
        &mut self,
        key: &[u8; 2],
        payload: &[u8],
    ) -> Result<(), WeirdoFileSystemError> {
        if payload.len() - BLOCK_META_DATA as usize > BLOCK_SIZE as usize {
            return Err(WeirdoFileSystemError::PayloadTooLarge);
        }
        let address_empty_block = self.addres_of_empty_block();
        let _ = self.storage.write(address_empty_block, &[0xff]);
        let _ = self.storage.write(address_empty_block + 1, key);
        let _ = self.storage.write(
            address_empty_block + 3,
            &(payload.len() as u16).to_le_bytes(),
        );
        let _ = self.storage.write(address_empty_block + 5, payload);
        self.empty_block = self.empty_block + 1;
        Ok(())
    }

    pub fn read_key_value(
        &mut self,
        key: &[u8; 2],
        value: &mut [u8; BLOCK_SIZE as usize - BLOCK_META_DATA as usize],
    ) -> Result<u16, WeirdoFileSystemError> {
        for block in 0..self.empty_block {
            let address = self.offset + (block * BLOCK_SIZE as u32) + 1;
            let mut stored_key = [0u8; 2];
            let _ = self.storage.read(address, &mut stored_key);
            if stored_key == *key {
                let mut stored_size = [0u8; 2];
                let _ = self.storage.read(address + 2, &mut stored_size);
                let _ = self.storage.read(address + 4, value);
                return Ok(u16::from_le_bytes(stored_size));
            }
        }
        Err(WeirdoFileSystemError::KeyNotFound)
    }

    fn addres_of_empty_block(&mut self) -> u32 {
        return self.offset + self.empty_block * BLOCK_SIZE as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_read_write() {
        let mut storage = MemoryStorage::new(0x100000);

        // Write some data to the storage
        let write_data = b"Hello, world!";

        match storage.write(0, write_data) {
            Ok(_) => {}
            Err(e) => panic!("Write operation failed"),
        }

        // Read the data back from the storage
        let mut read_buffer: Vec<u8> = alloc::vec![0; write_data.len()];
        match storage.read(0, &mut read_buffer) {
            Ok(_) => {
                assert_eq!(read_buffer.as_slice(), write_data);
            }
            Err(e) => panic!("Read operation failed"),
        }
    }

    #[test]
    fn test_memory_storage_capacity() {
        let mut storage = MemoryStorage::new(0x100000);
        assert_eq!(storage.capacity(), 0x100000); // Ensure the data vector has the correct size
    }

    #[test]
    fn test_fs_storage_read_write() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> = WeirdoFileSystem::new(storage, 0, 0x100000);

        let payload = include_bytes!("../examples/image_layout/patches/ebass.lwp");
        let size = payload.len();

        filesystem.write_key_value(&[0, 1], payload).unwrap();

        let mut buffer: [u8; 2042] = [0; 2042];
        let size_of_value = filesystem.read_key_value(&[0, 1], &mut buffer).unwrap();
        assert_eq!(size_of_value, size as u16);
        assert_eq!(&buffer[..size_of_value as usize], payload);
    }

    #[test]
    fn test_fs_free_blocks() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> = WeirdoFileSystem::new(storage, 0, 0x100000);

        let payload = include_bytes!("../examples/image_layout/patches/ebass.lwp");
        filesystem.write_key_value(&[0, 1], payload).unwrap();
        filesystem.write_key_value(&[0, 2], payload).unwrap();
        filesystem.write_key_value(&[0, 3], payload).unwrap();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
        filesystem.build_cache();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
    }
}
