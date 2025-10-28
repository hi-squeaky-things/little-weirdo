// Import necessary standard library components
use std::{
    fs::{read_dir, File},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

// Import the Hound crate for WAV file handling
use hound::WavReader;

// Define constants for soundbank configuration
const SOUNDBANK_ITEMS: usize = 1;
const SOUNDBANK_ORIGINAL: [&str; SOUNDBANK_ITEMS] =
    ["examples/soundbank/soundbank_pure_elektro/original"];
const SOUNDBANK_OUTPUT: [&str; SOUNDBANK_ITEMS] = ["examples/soundbank/soundbank_pure_elektro/src"];

// Main function - entry point of the program
fn main() {
    // Loop through each soundbank item defined
    for i in 0..SOUNDBANK_ITEMS {
        // Get the original soundbank source path
        let soundbank_original_source_path = Path::new(SOUNDBANK_ORIGINAL[i]);
        // Get the output soundbank source path
        let soundbank_source_path = Path::new(SOUNDBANK_OUTPUT[i]);

        // Initialize a counter for tracking processed files
        let mut counter = 0;

        // Read directory entries and collect them into a vector
        let mut paths: Vec<_> = read_dir(soundbank_original_source_path)
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        // Sort directory entries by filename for consistent processing
        paths.sort_by_key(|dir| dir.file_name());

        // Create a README.md file to document the soundbank
        let soundbank_reference_file_name = Path::new(soundbank_source_path).join("README.md");
        let mut soundbank_reference_file =
            File::create(soundbank_reference_file_name).expect("Failed to open file");

        // Get current timestamp for documentation
        let now = SystemTime::now();
        let _since_unix_epoch = now.duration_since(UNIX_EPOCH).unwrap();

        // Write initial markdown header to README
        let _ = soundbank_reference_file.write(format!("# Soundbank \n\n").as_bytes());

        // Write table header for the soundbank documentation
        let _ = soundbank_reference_file
            .write("| Index | Original | Output | Samples | Size | \n|-|-|-|-|-|\n".as_bytes());

        // Process each file in the directory
        for entry in paths {
            // Format the original source name for the README table
            let orginal_src_name = format!(
                "| {} | {:?} | \"wav{:?}.lwt\" | 600 | 1200 |\n",
                counter,
                entry.file_name(),
                counter
            );

            // Write the formatted line to the README file
            let _ = soundbank_reference_file.write(orginal_src_name.as_bytes());

            // Create the output filename for the raw audio data
            let soundbank_source_file_name = format!("wav{:?}.raw", counter);
            let soundbank_source =
                Path::new(soundbank_source_path).join(soundbank_source_file_name);

            // Create the output file for raw audio data
            let mut soundbank_source_file =
                File::create(soundbank_source).expect("Failed to open file");

            // Print debug information about the current file being processed
            println!("Process file = {:?}", entry);

            // Open the WAV file for reading
            let mut open = WavReader::open(entry.path()).unwrap();

            // Print the duration of the WAV file
            println!("Amount of sample = {:?}", open.duration());

            // Iterate through all samples in the WAV file
            for sample in open.samples::<i16>().into_iter() {
                // Extract the sample value
                let output = sample.unwrap();

                // Convert the sample to little-endian byte representation and write to output file
                let _ = soundbank_source_file.write(&output.to_ne_bytes());
            }

            // Increment the counter for next iteration
            counter = counter + 1;
        }
    }
}
