use little_weirdo::synth::patch::Patch;
use std::fs::{self, File};
use std::io::Write;

fn main() {
    // Load a synth patch from a JSON file
    let patch_dir = "examples/soundbank/synth/patches/original";
    let output_dir = "examples/soundbank/synth/patches/src";

    // Read all files from the patches directory
    let entries = fs::read_dir(patch_dir).expect("Failed to read directory");

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Only process JSON files
        if let Some(extension) = path.extension() {
            if extension == "json" {
                // Read the JSON file
                let json_content = fs::read_to_string(&path).expect("Failed to read JSON file");

                // Parse JSON
                let patch: Patch = serde_json::from_str(&json_content).unwrap();
                // Serialize to postcard format
                let mut buf = [0u8; 256];
                let serialized =
                    postcard::to_slice(&patch, &mut buf).expect("Failed to serialize to postcard");

                // Create output filename (same name, different extension)
                let filename = path.file_stem().unwrap().to_str().unwrap().to_string() + ".lwp";
                let output_path = std::path::Path::new(output_dir).join(filename);
                println!("{:?}", output_path);
                // Write to file
                let mut postcard_patch_file =
                    File::create(&output_path).expect("Failed to create output file");
                postcard_patch_file
                    .write_all(&serialized)
                    .expect("Failed to write to file");

                println!("Converted {} to {}", path.display(), output_path.display());
            }
        }
    }
}
