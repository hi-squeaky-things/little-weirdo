use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

use serde::{Deserialize, Serialize};

/// Selects the noise source used by a waveform oscillator.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseKind {
    /// The oscillator reads from its configured wavetable.
    #[default]
    None,
    /// Unfiltered random samples.
    White,
    /// Low-frequency-emphasized filtered random samples.
    Pink,
    /// Integrated random steps with bounded drift.
    Brown,
}

/// Integer-only noise generator for embedded audio paths.
pub struct NoiseGenerator {
    kind: NoiseKind,
    random: SmallRng,
    pink_state: i32,
    brown_state: i32,
}

impl NoiseGenerator {
    pub fn new(kind: NoiseKind, seed: u64) -> Self {
        Self {
            kind,
            random: SmallRng::seed_from_u64(seed),
            pink_state: 0,
            brown_state: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pink_state = 0;
        self.brown_state = 0;
    }

    pub fn set_kind(&mut self, kind: NoiseKind) {
        self.kind = kind;
        self.reset();
    }

    pub fn sample(&mut self) -> i16 {
        match self.kind {
            NoiseKind::None => 0,
            NoiseKind::White => self.random_sample(),
            NoiseKind::Pink => self.pink_sample(),
            NoiseKind::Brown => self.brown_sample(),
        }
    }

    fn random_sample(&mut self) -> i16 {
        self.random
            .random_range((i16::MIN + 1000)..(i16::MAX - 1000))
    }

    fn pink_sample(&mut self) -> i16 {
        let white = self.random_sample() as i32;
        self.pink_state += (white - self.pink_state) >> 5;
        self.pink_state = self.pink_state.clamp(i16::MIN as i32, i16::MAX as i32);
        self.pink_state as i16
    }

    fn brown_sample(&mut self) -> i16 {
        let step = (self.random_sample() as i32) >> 9;
        self.brown_state += step;
        self.brown_state -= self.brown_state >> 10;
        self.brown_state = self.brown_state.clamp(i16::MIN as i32, i16::MAX as i32);
        self.brown_state as i16
    }
}
