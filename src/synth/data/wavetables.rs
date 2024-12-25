#[derive(Copy, Clone, Debug)]
pub struct Wavetable {
    // 600 samples  = 1200 bytes = 1.2 Kb
    pub data: [u8; 1_200]
}

#[derive(Copy, Clone, Debug)]
pub struct SoundBank {
    // 10 x 1.2Kb = 12Kb per Soundbank
    pub wavetables: [Wavetable; 10]
}

