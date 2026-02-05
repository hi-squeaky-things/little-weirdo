pub const AMOUNT_OF_STEPS: usize = 16; // Number of steps per lane (e.g. 16th notes in a bar)
pub const AMOUNT_OF_LANES: usize = 5;  // Number of lanes (e.g. drum voices)

/// Main sequencer struct
pub struct Sequencer {
    sample_rate: u32,                  // Audio sample rate (Hz)
    bpm: u32,                          // Beats per minute
    lanes: [SequencerLane; AMOUNT_OF_LANES], // Array of lanes
    playing: bool,                     // Is the sequencer running?
    counter: u8,                       // Current step index
    sample_acc: u32,                   // Sample accumulator for timing
}

/// Represents a single lane (e.g. drum voice)
#[derive(Copy, Clone)]
pub struct SequencerLane {
    note: u8,                          // MIDI note number
    steps: [bool; AMOUNT_OF_STEPS],    // Step triggers (true = play note)
}

impl Sequencer {
    /// Create a new sequencer with given sample rate and bpm
    pub fn new(sample_rate: u32, bpm: u32) -> Self {
        Sequencer {
            sample_rate,
            bpm,
            lanes: [SequencerLane { note: 0, steps: [false; AMOUNT_OF_STEPS] }; AMOUNT_OF_LANES],
            playing: false,
            counter: 0,
            sample_acc: 0,
        }
    }

    /// Set the MIDI note for a lane
    pub fn set_lane_note(&mut self, lane: usize, note: u8) {
        if lane < AMOUNT_OF_LANES {
            self.lanes[lane].note = note;
        }
    }

    /// Activate a step in a lane
    pub fn set_step(&mut self, lane: usize, step: usize) {
        if step < 16 && lane < AMOUNT_OF_LANES {
            self.lanes[lane].steps[step] = true;
        }
    }

    /// Deactivate a step in a lane
    pub fn clear_step(&mut self, lane: usize, step: usize) {
        if step < 16 && lane < AMOUNT_OF_LANES {
            self.lanes[lane].steps[step] = false;
        }
    }

    /// Set the BPM (tempo)
    pub fn set_bpm(&mut self, bpm: u32) {
        self.bpm = bpm;
    }

    /// Get the current BPM
    pub fn get_bpm(&self) -> u32 {
        self.bpm
    }

    /// Start the sequencer
    pub fn start(&mut self) {
       self.playing = true;
    }

    /// Stop the sequencer
    pub fn stop(&mut self) {
       self.playing = false;
    }

    /// Called once per audio sample.
    /// Returns an array for each lane: (should trigger, note number).
    /// Timing is calculated based on sample rate and BPM.
    pub fn clock(&mut self) -> [(bool, u8); AMOUNT_OF_LANES]  {
        // If not playing or BPM is zero, return no triggers
        if !self.playing { return [(false, 0); AMOUNT_OF_LANES]; }
        if self.bpm == 0 { return [(false, 0); AMOUNT_OF_LANES]; }

        // Calculate how many samples per step (16th note)
        // samples per quarter note = sample_rate * 60 / bpm
        // 16 steps per bar (4 beats), so each step is a 16th note = quarter note / 4
        let samples_per_step = ((self.sample_rate as u64 * 60) / (self.bpm as u64 * 4)).max(1) as u32;

        // Increment sample accumulator
        self.sample_acc = self.sample_acc.wrapping_add(1);

        // If enough samples have passed, advance to next step
        if self.sample_acc >= samples_per_step {
            self.sample_acc = 0;
            let idx = (self.counter as usize) % AMOUNT_OF_STEPS;
            let mut hits = [(false, 0); AMOUNT_OF_LANES];
            // For each lane, check if the step is active and return note
            for i in 0..AMOUNT_OF_LANES {
                hits[i] = (self.lanes[i].steps[idx], self.lanes[i].note);
            }
            // Advance step counter
            self.counter = (self.counter.wrapping_add(1)) % (AMOUNT_OF_STEPS as u8);
            return hits;
        }
        // Otherwise, return no triggers
        [(false, 0); AMOUNT_OF_LANES]
    }
}