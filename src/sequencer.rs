pub const AMOUNT_OF_STEPS: usize = 16; // Number of steps per lane (e.g. 16th notes in a bar)
pub const AMOUNT_OF_LANES: usize = 10;  // Number of lanes (e.g. drum voices)

/// Main sequencer struct
pub struct Sequencer {
    lanes: [SequencerLane; AMOUNT_OF_LANES], // Array of lanes
    playing: bool,                     // Is the sequencer running?
    pub counter: u8,                       // Current step index
}

/// Represents a single lane (e.g. drum voice)
#[derive(Copy, Clone)]
pub struct SequencerLane {
    note: u8,                          // MIDI note number
    pub steps: [bool; AMOUNT_OF_STEPS],    // Step triggers (true = play note)
}

impl Sequencer {
    pub fn new() -> Self {
        Sequencer {
            lanes: [SequencerLane { note: 0, steps: [false; AMOUNT_OF_STEPS] }; AMOUNT_OF_LANES],
            playing: false,
            counter: 0,
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

    pub fn flip_step(&mut self, lane: usize, step: usize) ->  bool {
         if step < 16 && lane < AMOUNT_OF_LANES {
            self.lanes[lane].steps[step] = !self.lanes[lane].steps[step] 
        }
         self.lanes[lane].steps[step]
    }

    /// Deactivate a step in a lane
    pub fn clear_step(&mut self, lane: usize, step: usize) {
        if step < 16 && lane < AMOUNT_OF_LANES {
            self.lanes[lane].steps[step] = false;
        }
    }

    pub fn get_lane(&mut self, lane: usize) -> SequencerLane {
        self.lanes[lane]
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

        // Increment sample accumulator
        self.counter = self.counter.wrapping_add(1);
        if self.counter == 16 {
            self.counter = 0;
        }

        // If enough samples have passed, advance to next step
        let idx = (self.counter as usize) % AMOUNT_OF_STEPS;
        let mut hits = [(false, 0); AMOUNT_OF_LANES];
        // For each lane, check if the step is active and return note
        for i in 0..AMOUNT_OF_LANES {
            hits[i] = (self.lanes[i].steps[idx], self.lanes[i].note);
        }
        // Advance step counter
    
        return hits;
    
    }
}