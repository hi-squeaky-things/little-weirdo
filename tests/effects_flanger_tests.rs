use little_weirdo::effects::{
    flanger::{Flanger, FlangerConfiguration},
    Effect,
};

fn config() -> FlangerConfiguration {
    FlangerConfiguration {
        enabled: true,
        base_delay_time_ms: 1,
        depth_ms: 0,
        rate_hz: 0,
        mix_percentage: 100,
    }
}

#[test]
fn flanger_is_bypassed_when_disabled() {
    let mut effect = Flanger::new(
        FlangerConfiguration {
            enabled: false,
            ..config()
        },
        1_000,
    );

    assert_eq!(effect.clock(1_000), 1_000);
    assert_eq!(effect.clock(-2_000), -2_000);
}

#[test]
fn flanger_applies_base_delay_and_wet_mix() {
    let mut effect = Flanger::new(config(), 1_000);

    assert_eq!(effect.clock(1_000), 0);
    assert_eq!(effect.clock(2_000), 1_000);
    assert_eq!(effect.clock(3_000), 2_000);
}

#[test]
fn flanger_blends_dry_and_delayed_signal() {
    let mut effect = Flanger::new(
        FlangerConfiguration {
            mix_percentage: 50,
            ..config()
        },
        1_000,
    );

    assert_eq!(effect.clock(1_000), 500);
    assert_eq!(effect.clock(2_000), 1_500);
}

#[test]
fn flanger_modulation_changes_delay_tap() {
    let mut effect = Flanger::new(
        FlangerConfiguration {
            base_delay_time_ms: 2,
            depth_ms: 1,
            rate_hz: 250,
            ..config()
        },
        1_000,
    );

    assert_eq!(effect.clock(1_000), 0);
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 1_000);
}

#[test]
fn flanger_saturates_mixed_signal() {
    let mut positive = Flanger::new(config(), 1_000);
    positive.clock(i16::MAX);
    assert_eq!(positive.clock(i16::MAX), i16::MAX);

    let mut negative = Flanger::new(config(), 1_000);
    negative.clock(i16::MIN);
    assert_eq!(negative.clock(i16::MIN), i16::MIN);
}

#[test]
fn flanger_reload_resets_buffer_and_phase() {
    let mut effect = Flanger::new(config(), 1_000);
    effect.clock(1_000);
    effect.clock(0);

    effect.reload(config(), 1_000);
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 0);

    effect.reset();
    assert_eq!(effect.clock(0), 0);
}

#[test]
fn flanger_handles_extreme_configuration_without_panicking() {
    let mut effect = Flanger::new(
        FlangerConfiguration {
            enabled: true,
            base_delay_time_ms: u16::MAX,
            depth_ms: u16::MAX,
            rate_hz: u16::MAX,
            mix_percentage: u8::MAX,
        },
        u16::MAX,
    );

    let _ = effect.clock(i16::MAX);
}
