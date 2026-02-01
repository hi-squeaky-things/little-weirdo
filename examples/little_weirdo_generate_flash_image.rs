use std::io::Read;
use std::{fs::File, io::Write};

use std::fs;

use little_weirdo::fs::{MemoryStorage, WeirdoFileSystem};

fn main() {
    let storage = MemoryStorage::new(0x100000);
    let mut filesystem: WeirdoFileSystem<MemoryStorage> = WeirdoFileSystem::new(storage, 0, 0x100000);
    filesystem.format();


     let entries = fs::read_dir("./examples/image_layout/patches").expect("Failed to read directory");

     let mut id = 0;
     let mut key:u16 = 100;
    for entry in entries {
        let mut buffer:[u8;2042] = [0;2042];
        let path = entry.unwrap().path();
        if path.is_file() {
            println!("file {:?} stored with id {:?}", path.file_name(), id);
           
            let mut file = fs::File::open(path).unwrap();
            let len = file.read(&mut buffer).unwrap();
             println!("file size = {:?}", len);
                key = key + id;
                filesystem.write_key_value(key, &buffer[0..len]);
                id = id + 1;
         
        }

    }

     let payload = include_bytes!("../examples/image_layout/samples/wav0.raw");
    let result = filesystem.write_key_value(900, payload).unwrap();



    
    // export to disk
    let mut data = filesystem.storage;
    let mut file = File::create("little_squeaky_machine_data_image.bin").expect("Unable to create file");
    file.write_all(data.dump()).expect("Unable to write data");
}
