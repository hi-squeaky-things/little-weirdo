use std::fs::File;
use std::path::Path;
use hound::{self, WavReader};
///
/// This program generates a list of static waveforms in the form of arrays of 16-bit integers.
///
/// It reads WAV files from the "data" directory and prints them out as `pub static` variables,
/// ready to be used in other parts of your project.
///
///
fn main() {
    type waveform = (String, String);
    let waveform_list: [waveform; 8] = [
        ("BASS".to_string(), "data/ebass.WAV".to_string()),
        ("SAWTOOTH".to_string(), "data/sawtooth.WAV".to_string()),
        ("SINE".to_string(), "data/sinus.WAV".to_string()),
        ("SQUARE".to_string(), "data/square.WAV".to_string()),
        ("SQUARE_10".to_string(), "data/square_10.WAV".to_string()),
        ("SQUARE_25".to_string(), "data/square_25.WAV".to_string()),
        ("TRIANGLE".to_string(), "data/triangle.WAV".to_string()),
        ("PIANO".to_string(), "data/piano.WAV".to_string()),
    ];

    for i in waveform_list {
        let open = WavReader::open(Path::new(&i.1));
        print!("pub static {}: [i16; 600] = [", &i.0);
        open.unwrap().samples::<i16>()
            .into_iter()
            .for_each(|sample| print!("{:?},", sample.unwrap()));
        print!("];");
    }
}
