
use pitch_calc;
/// This program calculates and prints the frequencies of all 127 semitone steps in the MIDI range (0-127).
///
/// The `pitch_calc` crate is used to calculate the frequency for each semitone step.
///
/// The output is a list of frequencies, where each frequency is represented as an integer (in Hz) corresponding to its semitone step number.

fn main() {
    print!("[");
   for i in 0..127 {
     let freq = pitch_calc::calc::hz_from_step(i as f32);         
     print!("{},", freq as i32);
   } ;
   print!("]");
}