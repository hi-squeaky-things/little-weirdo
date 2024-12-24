/// auto generated using examples/create_wave_table.rs

#[derive(Copy, Clone, Debug)]
pub struct Wavetable {
    pub data: [u8; 1200]
}

pub struct SoundBank {
    pub wavetables: [Wavetable; 8]
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
    ],
};

