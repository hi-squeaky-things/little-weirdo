```
 __   ____  ____  __      _  _  ____  __  ____  ____   __  
(  ) (_  _)(_  _)(  )    / )( \(  __)(  )(  _ \(    \ /  \ 
/ (_/\ )(    )(  / (_/\  \ /\ / ) _)  )(  )   / ) D ((  O )
\____/(__)  (__) \____/  (_/\_)(____)(__)(__\_)(____/ \__/ 
```

A Rust #no-std optimized wave table synthesizer for embedded devices.

## Tools

[`create_wave_table.rs`](/examples/create_wave_table.rs) This program generates a list of static waveforms in the form of arrays of 16-bit integers by reading WAV files from the "data" directory and printing them out as pub static variables. It's to be used to pre-compute and store waveforms that can be reused later.

[`streaming-synth.rs`](/examples/streaming-synth.rs) This program demonstrates how to use CPAL (Cross Platform Audio Library) to create a synthesizer that generates sound in real-time using Little Weirdo. 

[`wavetable_freq_lookup.rs`](/examples/wavetable_freq_lookup.rs) This program generates a list of static arrays representing frequency lookups, which can be used in wave table synthesis to map note frequencies to corresponding waveform indices. It calculates these arrays based on the sample rate and frequency values.

[`create_note_to_frequency.rs`](/examples/create_note_to_freq_table.rs) This program calculates and prints the frequencies of all 127 semitone steps in the MIDI range (0-127). The output is a list of frequencies, where each frequency is represented as an integer (in Hz) corresponding to its semitone step number.
