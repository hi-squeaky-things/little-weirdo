use std::{fs::{read_dir, DirEntry, File}, io::Write, path::Path};

use hound::WavReader;

const  SOUNDBANK_ORIGINAL: &str = "./data/soundbank_0/original";
const  SOUNDBANK_OUTPUT: &str = "./data/soundbank_0/src";

fn main() {
    let soundbank_original_source_path = Path::new(SOUNDBANK_ORIGINAL);
    let soundbank_source_path = Path::new(SOUNDBANK_OUTPUT);

    let mut counter = 0;
   for file in read_dir(soundbank_original_source_path).unwrap() {
        match file {
            Ok(dir) => {

                let soundbank_source_file_name = format!("wav{:?}.lwt", counter);
                let soundbank_source  = Path::new(soundbank_source_path).join(soundbank_source_file_name);
                let mut soundbank_source_file = File::create(soundbank_source).expect("Failed to open file");

                println!("Process file = {:?}", dir);
                let mut open = WavReader::open(dir.path()).unwrap();
                println!("Amount of sample = {:?}", open.duration());
                for sample in open.samples::<i16>()
                    .into_iter() {
                        let output = sample.unwrap();
                        soundbank_source_file.write(&output.to_ne_bytes());
                    }
                counter = counter + 1; 
            },
            Err(err) => {
                println!("err = {:?}", err);
            } ,
        }
    }

}

