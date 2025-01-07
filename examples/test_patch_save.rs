use std::{fs::File, io::{Read, Write}, path::Path};

use little_weirdo::synth::patch::Patch;
use little_weirdo_soundbanks::soundbanks::{SOUND_BANK_PURE_ELEKTRO, SOUND_BANK_PURPLE_WAVES};
use postcard::{from_bytes, to_vec};
use serde::Serialize;

fn main() {
    let patch: Patch = SOUND_BANK_PURE_ELEKTRO.patches[1];
    println!("{:?}",patch.synth_config );
    let mut buf = [0u8; 163];

    match postcard::to_slice(&patch, &mut buf) {
        Ok(result) => {println!("Serialize Ok")},
        Err(error) => {println!("{:?}", error)},
    }

    let mut file_name = Path::new("./").join("patch.p");
    let mut file =  File::create(file_name).expect("Failed to open file");
   file.write(&buf);


    let mut buf2 = [0u8; 163];
    file.read(&mut buf2);

   let test:Patch = postcard::from_bytes(&buf2).unwrap();
   println!("{:?}",test.synth_config );
}