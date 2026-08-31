use cpal::Sample; // For audio sample conversion
use little_weirdo::{
    synth::{
        self,
        data::{
            patches::{BoxedPatch, BoxedPatches, Patches},
            wavetables::{BoxedWavetable, BoxedWavetables},
        },
        patch::Patch,
    },
};
use std::{
    fs::{self, read_dir, File},
    io::BufReader,
    sync::Arc,
}; // File system operations and thread-safe reference counting

// Constants defining audio parameters
const SAMPLE_RATE: u16 = 44_100; // Audio sample rate in Hz
const CLIPPING: u16 = 32_000; // Threshold for detecting clipping
const HEADROOM: f64 = -12.0; // Minimum acceptable decibel level

fn main() {
    // Configure WAV file specification for output
    let spec = hound::WavSpec {
        channels: 1,                             // Mono audio
        sample_rate: 44100,                      // Same as our constant
        bits_per_sample: 16,                     // 16-bit samples
        sample_format: hound::SampleFormat::Int, // Integer format
    };


    // Initialize wavetables storage on heap
    let mut wt_on_heap = BoxedWavetables::new();

    // Load 10 wavetables from files
    for id in 0..55 {
        let filename = format!("examples/soundbank/waveforms/src/wav{}.raw", id);
        let contents = fs::read(filename).unwrap(); // Read file contents
        let bytes: &[u8] = &contents; // Convert to byte slice
        wt_on_heap.add(BoxedWavetable::new(bytes)); // Add to wavetables collection
    }

    // Create an Arc (thread-safe reference) to the wavetables
    let wt = Arc::new(wt_on_heap);

    let path = "examples/soundbank/waveforms/patches/orginal".to_string();
    let mut paths: Vec<_> = read_dir(path).unwrap().filter_map(Result::ok).collect();
    let mut patches = BoxedPatches::new();
    // Sort directory entries by filename for consistent processing
    paths.sort_by_key(|dir| dir.file_name());
    for entry in paths {
        let path = entry.path();
        if path.is_file() {
            println!("{:?}", path.file_name());
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);

            let patch: Patch = serde_json::from_reader(reader).unwrap();
            let patch_box = BoxedPatch::new(patch);
            // let bytes: &[u8] = &contents;
            //   samples_on_heap.add(BoxedPatch::new(contents));
            patches.add(patch_box);
        }
    }

    //let patch =  patches.get_patches_reference(0);

    let amount_of_patches = patches.size();
    let patches_heap = Arc::new(patches);

    // Create a new synthesizer instance with specified parameters
    let mut synth: synth::Synth =
        synth::Synth::new(44100, 2, Arc::clone(&patches_heap), Arc::clone(&wt));

    // Array to track clipping occurrences
    let mut clipped: [i32; 2] = [0; 2];

    // Test notes from MIDI note 24 to 69 (about 5 octaves)
    for patch_index in 0..amount_of_patches {
        synth.load_patch(patch_index as u8);
        let active_patch = &patches_heap.get_patches_reference(patch_index as u8);
        println!("{:}", active_patch.name);
    // Create WAV writer to output audio data
    let mut writer = hound::WavWriter::create(format!("patch_{:}.wav", active_patch.name), spec).unwrap();
        for _n in 0..SAMPLE_RATE/4 {
            synth.clock_and_output();
        }

        for note in 0..4 {
            // Turn on a note with velocity 100
            synth.note_on(36 + note * 12, 100);

            let mut total: f64 = 0.0; // Accumulator for RMS calculation

            // Generate audio samples for one second per note
            for _n in 0..SAMPLE_RATE/4 {
                // Get output from synthesizer (stereo)
                let output = synth.clock_and_output();

                // Write left channel sample to WAV file
                writer.write_sample(output[0]).unwrap();

                // Convert sample to f64 for RMS calculation
                let left: f64 = Sample::from_sample(output[0]);
                total += left * left; // Add squared sample to total

                // Check for clipping in left channel
                if output[0].abs() > CLIPPING as i16 {
                    clipped[0] = clipped[0] + 1;
                }

                // Check for clipping in right channel (though we're using mono)
                if output[1].abs() > CLIPPING as i16 {
                    clipped[0] = clipped[0] + 1;
                }
            }

            // Turn off the note
            synth.note_off(36 + note * 12);

            // Generate more samples while note is off (release phase)
            for _n in 0..SAMPLE_RATE/4 {
                let output = synth.clock_and_output();
                writer.write_sample(output[0]).unwrap();
            }

            // Calculate RMS (Root Mean Square) value
            let rms = total / SAMPLE_RATE as f64;

            // Convert RMS to decibels
            let db = decibel(rms).round();

            // Print note number, decibel level, and status indicators
            print!("[{:?}]::", note);
            print!("{:?}::", db);

            // Print "H" if volume exceeds headroom threshold
            if db > HEADROOM {
                print!("H")
            }

            // Print "C" if clipping occurred, "." otherwise
            if clipped[0] > 0 {
                print!("C");
            } else {
                print!(".");
            }

            // Print separator
            print!("-");

            // Print newline every 8 notes for readability
            if note % 8 == 0 {
                println!("");
            }
        }

        // Print final newline
        println!("");
    }
}

// Function to convert RMS amplitude to decibel scale
fn decibel(rms: f64) -> f64 {
    // Convert RMS value to decibels using a simple formula
    10.0 * rms.log10()
}
