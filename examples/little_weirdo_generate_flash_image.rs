
use std::{fs::File, io::Write, ops::Range};

use embedded_storage_inmemory::MemFlash;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use sequential_storage::{cache::{KeyPointerCache, NoCache}, mock_flash};
use tokio::sync::Mutex;

const MAP_FLASH_RANGE: Range<u32> = 0..(0x20000);

type MockFlash = mock_flash::MockFlashBase<1024, 4, 64>;
  

#[tokio::main]
async fn main() {
     let storage = MockFlash::default();
     println!("length = {:?}", storage.as_bytes().len());
     

    
    let mut data_buffer: [u8; 0x10] = [0u8; 0x10];

    let mut map_storage = sequential_storage::map::MapStorage::<u16, _, _>::new(
        storage,
        const { sequential_storage::map::MapConfig::new(MAP_FLASH_RANGE) },
        NoCache
    );

    

   let result = map_storage
        .store_item(
            &mut data_buffer,
            &1,
            &69)
        .await;
    match result {
        Ok(())=>{println!("Successfully stored item with key 1");}
        Err(e) => {println!("Error storing item: {:?}", e);},
    };
    
    let data = map_storage.flash().as_bytes();
    let mut file = File::create("output.bin").expect("Unable to create file");
    file.write_all(data).expect("Unable to write data");

 
}
