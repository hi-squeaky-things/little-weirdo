use little_weirdo::effects::{
    overdrive::{KindOfOverdrive, Overdrive, OverdriveConfiguration},
    Effect,
};

fn configuration(gain_percentage: u8) -> OverdriveConfiguration {
    OverdriveConfiguration {
        threshold: 1_000,
        kind: KindOfOverdrive::Hard,
        enabled: true,
        gain_percentage,
    }
}

#[test]
fn overdrive_gain_zero_keeps_processed_amplitude_unchanged() {
    let mut overdrive = Overdrive::new(configuration(0));

    assert_eq!(overdrive.clock(2_000), 1_000);
}

#[test]
fn overdrive_gain_hundred_doubles_processed_amplitude() {
    let mut overdrive = Overdrive::new(configuration(100));

    assert_eq!(overdrive.clock(2_000), 2_000);
}

#[test]
fn overdrive_gain_is_clamped_to_i16_range() {
    let mut overdrive = Overdrive::new(configuration(100));

    assert_eq!(overdrive.clock(i16::MAX), 2_000);
}

#[test]
fn disabled_overdrive_does_not_apply_makeup_gain() {
    let mut config = configuration(100);
    config.enabled = false;
    let mut overdrive = Overdrive::new(config);

    assert_eq!(overdrive.clock(2_000), 2_000);
}