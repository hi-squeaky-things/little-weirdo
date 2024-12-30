use crate::synth::patch::Patch;

#[derive(Copy, Clone)]
pub struct Wavetable {
    // 600 samples  = 1200 bytes = 1.2 Kb
    pub data: &'static[u8; 1_200]
}

#[derive(Copy, Clone)]
pub struct SoundBank {
    // 10 x 1.2Kb = 12Kb per Soundbank
    pub patches: [Patch; 2],
    pub wavetables: [Wavetable; 10]
}
