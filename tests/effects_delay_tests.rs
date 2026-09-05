use little_weirdo::effects::{
    delay::{Delay, DelayConfiguration},
    Effect,
};

#[test]
fn delay_applies_mixed_signal_when_delay_time_is_reached() {
    let mut effect = Delay::new(
        DelayConfiguration {
            enabled: true,
            delay_time: 1,
            delay_decrease_percentage: 0,
            mix_percentage: 50,
            feedback: false,
            feedback_percentage: 0,
        },
        1000,
    );

    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(2000), 2000);
    assert_eq!(effect.clock(3000), 3500);
    assert_eq!(effect.clock(4000), 5000);
}

#[test]
fn delay_is_bypassed_when_disabled() {
    let mut effect = Delay::new(
        DelayConfiguration {
            enabled: false,
            delay_time: 1,
            delay_decrease_percentage: 0,
            mix_percentage: 50,
            feedback: false,
            feedback_percentage: 0,
        },
        1000,
    );

    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(2000), 2000);
    assert_eq!(effect.clock(3000), 3000);
}

#[test]
fn delay_feedback_reinserts_processed_signal() {
    let mut effect = Delay::new(
        DelayConfiguration {
            enabled: true,
            delay_time: 1,
            delay_decrease_percentage: 0,
            mix_percentage: 50,
            feedback: true,
            feedback_percentage: 50,
        },
        1000,
    );
    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(2000), 2000);
    assert_eq!(effect.clock(3000), 3500);
    assert_eq!(effect.clock(4000), 5000);
}

#[test]
fn delay_time_is_converted_from_milliseconds_using_sample_rate() {
    let mut effect = Delay::new(
        DelayConfiguration {
            enabled: true,
            delay_time: 1,
            delay_decrease_percentage: 0,
            mix_percentage: 50,
            feedback: false,
            feedback_percentage: 0,
        },
        2000,
    );

    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(2000), 2000);
    assert_eq!(effect.clock(3000), 3000);
    assert_eq!(effect.clock(4000), 4500);
}

#[test]
fn delay_saturates_signal_mix_instead_of_overflowing() {
    let mut positive_effect = Delay::new(
        DelayConfiguration {
            enabled: true,
            delay_time: 1,
            delay_decrease_percentage: 0,
            mix_percentage: 100,
            feedback: false,
            feedback_percentage: 0,
        },
        1000,
    );
    assert_eq!(positive_effect.clock(i16::MAX), i16::MAX);
    assert_eq!(positive_effect.clock(i16::MAX), i16::MAX);
    assert_eq!(positive_effect.clock(i16::MAX), i16::MAX);

    let mut negative_effect = Delay::new(
        DelayConfiguration {
            enabled: true,
            delay_time: 1,
            delay_decrease_percentage: 0,
            mix_percentage: 100,
            feedback: false,
            feedback_percentage: 0,
        },
        1000,
    );
    assert_eq!(negative_effect.clock(i16::MIN), i16::MIN);
    assert_eq!(negative_effect.clock(i16::MIN), i16::MIN);
    assert_eq!(negative_effect.clock(i16::MIN), i16::MIN);
}

#[test]
fn delay_does_not_decrease_during_initial_delay_cycle() {
    let mut effect = Delay::new(
        DelayConfiguration {
            enabled: true,
            delay_time: 4,
            delay_decrease_percentage: 50,
            mix_percentage: 100,
            feedback: true,
            feedback_percentage: 100,
        },
        1000,
    );

    assert_eq!(effect.clock(1000), 1000);
    for _ in 0..4 {
        assert_eq!(effect.clock(0), 0);
    }
    assert_eq!(effect.clock(0), 1000);
}

#[test]
fn delay_keeps_delay_time_fixed_while_feedback_decreases() {
    let mut effect = Delay::new(
        DelayConfiguration {
            enabled: true,
            delay_time: 2,
            delay_decrease_percentage: 50,
            mix_percentage: 100,
            feedback: true,
            feedback_percentage: 100,
        },
        1000,
    );

    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 1000);

    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 1000);
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(0), 500);
}

