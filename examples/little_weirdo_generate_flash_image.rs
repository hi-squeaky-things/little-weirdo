use std::io::Read;
use std::{fs::File, io::Write};

use std::fs::{self, read_dir};

use little_weirdo::fs::{MemoryStorage, WeirdoFileSystem};

fn main() {
    let storage = MemoryStorage::new(0x100000);
    let mut filesystem: WeirdoFileSystem<MemoryStorage> =
        WeirdoFileSystem::new(storage, 0, 0x100000);
    filesystem.format();

    store_items(
        &mut filesystem,
        "./examples/image_layout/patches".to_string(),
        100,
    );
    store_items(
        &mut filesystem,
        "./examples/image_layout/waveforms".to_string(),
        700,
    );
    store_items(
        &mut filesystem,
        "./examples/image_layout/samples".to_string(),
        900,
    );

    // export to disk
    let mut data = filesystem.storage;
    let mut file =
        File::create("little_squeaky_machine_data_image.bin").expect("Unable to create file");
    file.write_all(data.dump()).expect("Unable to write data");
}

fn store_items(filesystem: &mut WeirdoFileSystem<MemoryStorage>, path: String, key: u16) {
    // Read directory entries and collect them into a vector
    let mut paths: Vec<_> = read_dir(path).unwrap().filter_map(Result::ok).collect();

    // Sort directory entries by filename for consistent processing
    paths.sort_by_key(|dir| dir.file_name());

    let mut key: u16 = key;
    for entry in paths {
        let mut buffer: [u8; 32000] = [0; 32000];
        let path = entry.path();
        if path.is_file() {
            println!("file {:?} stored with key {:?}", path.file_name(), key);

            let mut file = fs::File::open(path).unwrap();
            let len = file.read(&mut buffer).unwrap();
            println!("file size = {:?}", len);
            filesystem.write_key_value(key, &buffer[0..len]);
            key = key + 1;
        }
    }
}
