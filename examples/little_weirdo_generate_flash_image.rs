use std::{fs::File, io::Write};

use little_weirdo::fs::{MemoryStorage, WeirdoFileSystem};

fn main() {
    let storage = MemoryStorage::new(0x100000);
    let mut filesystem: WeirdoFileSystem<MemoryStorage> = WeirdoFileSystem::new(storage, 0, 0x100000);

    filesystem.write_key_value(&[0,1], include_bytes!("../examples/image_layout/patches/ebass.lwp")).unwrap();
    filesystem.write_key_value(&[0,2], include_bytes!("../examples/image_layout/patches/piano.lwp")).unwrap();
    filesystem.write_key_value(&[1,3], include_bytes!("../examples/image_layout/waveforms/wav0.raw")).unwrap();
    //filesystem.write_key_value(&[2,1], include_bytes!("../examples/image_layout/samples/wav0.raw")).unwrap();

    let mut buffer:[u8;2042] = [0;2042];
    filesystem.read_key_value(&[0,1],&mut buffer).unwrap();
    // export to disk
    let mut data = filesystem.storage;
    let mut file = File::create("little_squeaky_machine_data_image.bin").expect("Unable to create file");
    file.write_all(data.dump()).expect("Unable to write data");
}
