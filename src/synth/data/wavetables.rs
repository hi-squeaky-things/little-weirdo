#[derive(Copy, Clone, Debug)]
pub struct Wavetable {
    // 600 samples  = 1200 bytes = 1.2 Kb
    pub data: [u8; 1_200]
}

pub struct SoundBank {
    // 10 x 1.2Kb = 12Kb per Soundbank
    pub wavetables: [Wavetable; 10]
}

pub static SOUND_BANK_0:SoundBank = SoundBank {
    wavetables: [
        Wavetable {
           data: *include_bytes!("../../../data/soundbank_0/src/wav0.lwt"),
        },
        Wavetable {
            data: *include_bytes!("../../../data/soundbank_0/src/wav1.lwt") 
        },
        Wavetable {
            data: *include_bytes!("../../../data/soundbank_0/src/wav2.lwt") 
         },
         Wavetable {
             data: *include_bytes!("../../../data/soundbank_0/src/wav3.lwt") 
         },
         Wavetable {
            data: *include_bytes!("../../../data/soundbank_0/src/wav4.lwt") 
         },
         Wavetable {
             data: *include_bytes!("../../../data/soundbank_0/src/wav5.lwt") 
         },
         Wavetable {
            data: *include_bytes!("../../../data/soundbank_0/src/wav6.lwt") 
         },
         Wavetable {
             data: *include_bytes!("../../../data/soundbank_0/src/wav7.lwt") 
         },
         Wavetable {
            data: *include_bytes!("../../../data/soundbank_0/src/wav8.lwt") 
        },
        Wavetable {
            data: *include_bytes!("../../../data/soundbank_0/src/wav9.lwt") 
        },
    ],
};

