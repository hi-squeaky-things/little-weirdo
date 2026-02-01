use core::iter;

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
    block_size: u16,
    payload_size: u16,
}

#[derive(Debug)]
pub enum WeirdoFileSystemError {
    PayloadTooLarge,
    KeyNotFound,
    KeyToLarge,
}

// block = 2048 = [[u8=status][u16=key][u16=size][u16=next_block][data]]
// [0] = 'E' (Empty) / 'O' (Occupied)
// [1..2] = key
// [3..4] = size of payload (size = 2041 if key next_block exists)
// [5..6] = key of next_block in chain
// [7..] = payload (max = 2048-7 = 2041 bytes)

pub enum BlockStatus {
    Empty = 'E' as isize,
    Occupied = 'O' as isize,
}

const OFFSET_BLOCK_STATUS: u8 = 0x00;
const OFFSET_ADDRESS_KEY: u8 = 0x01;
const OFFSET_ADDRESS_SIZE: u8 = 0x03;
const OFFSET_ADDRESS_NEXT: u8 = 0x05;
const OFFSET_ADDRESS_PAYLOAD: u8 = 0x07;
const MAX_KEY_ID: u16 = 999;
const BLOCK_SIZE: u16 = 2048;

impl<T> WeirdoFileSystem<T>
where
    T: Storage,
{
    pub fn new(storage: T, offset: u32, size: u32) -> Self {
        let mut new_fs = WeirdoFileSystem {
            storage,
            offset,
            size,
            empty_block: 0,
            total_blocks: 0,
            block_size: BLOCK_SIZE,
            payload_size: BLOCK_SIZE - OFFSET_ADDRESS_PAYLOAD as u16,
        };
        new_fs.build_cache();
        new_fs
    }

    pub fn format(&mut self) {
        for block in 0..self.total_blocks {
            let block_address = self.offset + (block * self.block_size as u32);
            let _ = self.storage.write(
                block_address + OFFSET_BLOCK_STATUS as u32,
                &[BlockStatus::Empty as u8],
            );
        }
        self.build_cache();
    }

    fn build_cache(&mut self) {
        self.empty_block = 0;
        self.total_blocks = (self.size as u32 / self.block_size as u32);
        for block in 0..self.total_blocks {
            let address = self.offset + (block * self.block_size as u32);
            let mut block_status = [0u8; 1];
            let _ = self
                .storage
                .read(address + OFFSET_BLOCK_STATUS as u32, &mut block_status);
            if block_status[0] == BlockStatus::Empty as u8 {
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
        key: u16,
        payload: &[u8],
    ) -> Result<(), WeirdoFileSystemError> {
        if key > MAX_KEY_ID {
            return Err(WeirdoFileSystemError::KeyToLarge);
        }
        //  if payload.len() + OFFSET_ADDRESS_PAYLOAD as usize >= self.block_size as usize {
        //      return Err(WeirdoFileSystemError::PayloadTooLarge);
        //  }

        let chunks = payload.chunks(self.payload_size as usize);

        let mut block_key = key;
        for (i, block) in chunks.enumerate() {
            let empty_block_address = self.addres_of_empty_block();
            let _ = self.storage.write(
                empty_block_address + OFFSET_BLOCK_STATUS as u32,
                &[BlockStatus::Occupied as u8],
            );
            let _ = self.storage.write(
                empty_block_address + OFFSET_ADDRESS_KEY as u32,
                &block_key.to_le_bytes(),
            );
            let _ = self.storage.write(
                empty_block_address + OFFSET_ADDRESS_SIZE as u32,
                &(block.len() as u16).to_le_bytes(),
            );
            if block.len() < self.payload_size as usize {
                let _ = self
                    .storage
                    .write(empty_block_address + OFFSET_ADDRESS_NEXT as u32, &[0xFF, 0xFF]);
            } else {
                block_key = block_key + 1000;
                let _ = self.storage.write(
                    empty_block_address + OFFSET_ADDRESS_NEXT as u32,
                    &block_key.to_le_bytes(),
                );
            }

            let _ = self
                .storage
                .write(empty_block_address + OFFSET_ADDRESS_PAYLOAD as u32, block);
            self.empty_block = self.empty_block + 1;
        }

        Ok(())
    }

    pub fn read_key_value(
        &mut self,
        key: u16,
        value: &mut [u8],
    ) -> Result<u16, WeirdoFileSystemError> {
        if key > MAX_KEY_ID {
            return Err(WeirdoFileSystemError::KeyToLarge);
        }

        let mut block_address: u32;
        let (found, found_block_address) = self.contains_key(key);
        if found {
            let mut payload_size: u16 = 0;
            let mut stored_size = [0u8; 2];
            let mut next_key = [0u8; 2];
            block_address = found_block_address;
            loop {
                let _ = self
                    .storage
                    .read(block_address + OFFSET_ADDRESS_SIZE as u32, &mut stored_size);
                let _ = self.storage.read(
                    block_address + OFFSET_ADDRESS_PAYLOAD as u32,
                    &mut value[payload_size as usize..],
                );
                let _ = self
                    .storage
                    .read(block_address + OFFSET_ADDRESS_NEXT as u32, &mut next_key);

                payload_size += u16::from_le_bytes(stored_size);
                let next_key_retreived = u16::from_le_bytes(next_key);

                if next_key_retreived == 0xFFFF {
                    break;
                }

                let (found, found_block_address) = self.contains_key(next_key_retreived);
                block_address = found_block_address;
                if !found {
                    return Err(WeirdoFileSystemError::KeyNotFound);
                }
            }

            return Ok(payload_size);
        } else {
            Err(WeirdoFileSystemError::KeyNotFound)
        }
    }

    fn contains_key(&mut self, key: u16) -> (bool, u32) {
        for block in 0..self.empty_block {
            let block_address = self.offset + (block * self.block_size as u32);
            let mut stored_key = [0u8; 2];
            let _ = self
                .storage
                .read(block_address + OFFSET_ADDRESS_KEY as u32, &mut stored_key);
            if stored_key == key.to_le_bytes() {
                return (true, block_address);
            }
        }
        return (false, 0);
    }

    fn addres_of_empty_block(&mut self) -> u32 {
        return self.offset + self.empty_block * self.block_size as u32;
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
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("../../examples/image_layout/patches/ebass.lwp");
        let size = payload.len();

        filesystem.write_key_value(1, payload).unwrap();

        let mut buffer: [u8; 2042] = [0; 2042];
        let size_of_value = filesystem.read_key_value(1, &mut buffer).unwrap();
        assert_eq!(size_of_value, size as u16);
        assert_eq!(&buffer[..size_of_value as usize], payload);
    }

    #[test]
    fn test_fs_free_blocks() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("../../examples/image_layout/patches/ebass.lwp");
        filesystem.write_key_value(1, payload).unwrap();
        filesystem.write_key_value(2, payload).unwrap();
        filesystem.write_key_value(3, payload).unwrap();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
        filesystem.build_cache();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
    }

    #[test]
    fn test_chunking() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("../../examples/image_layout/samples/wav0.raw");
        let result = filesystem.write_key_value(800, payload).unwrap();
         let result = filesystem.write_key_value(801, payload).unwrap();

        let mut buffer: [u8; 30_000] = [0; 30_000];
        let size_of_value = filesystem.read_key_value(800, &mut buffer).unwrap();
        assert_eq!(size_of_value, payload.len() as u16);
        assert_eq!(payload, &buffer[..size_of_value as usize]);
    }

    #[test]
    fn test_fs_format() {
        let storage = MemoryStorage::new(0x100000);
        let mut filesystem: WeirdoFileSystem<MemoryStorage> =
            WeirdoFileSystem::new(storage, 0, 0x100000);
        filesystem.format();

        let payload = include_bytes!("../../examples/image_layout/patches/ebass.lwp");
        filesystem.write_key_value(1, payload).unwrap();
        filesystem.write_key_value(2, payload).unwrap();
        filesystem.write_key_value(3, payload).unwrap();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 509);
        filesystem.format();
        let free_blocks = filesystem.amount_of_free_blocks();
        assert_eq!(free_blocks, 512);
    }
}
