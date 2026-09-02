use little_weirdo::effects::{
    delay::{Delay, DelayConfiguration},
    Effect,
};

#[test]
fn delay_applies_mixed_signal_when_delay_time_is_reached() {
    let mut effect = Delay::new(DelayConfiguration {
        enabled: true,
        delay_time: 1,
        mix_percentage: 50,
        feedback: false,
        feedback_percentage: 0,
    });

    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(2000), 2000);
    assert_eq!(effect.clock(3000), 3500);
    assert_eq!(effect.clock(4000), 5000);
}

#[test]
fn delay_is_bypassed_when_disabled() {
    let mut effect = Delay::new(DelayConfiguration {
        enabled: false,
        delay_time: 1,
        mix_percentage: 50,
        feedback: false,
        feedback_percentage: 0,
    });

    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(2000), 2000);
    assert_eq!(effect.clock(3000), 3000);
}

#[test]
fn delay_feedback_reinserts_processed_signal() {
    let mut effect = Delay::new(DelayConfiguration {
        enabled: true,
        delay_time: 1,
        mix_percentage: 50,
        feedback: true,
        feedback_percentage: 50,
    });
    assert_eq!(effect.clock(1000), 1000);
    assert_eq!(effect.clock(2000), 2000);
    assert_eq!(effect.clock(3000), 3500);
    assert_eq!(effect.clock(4000), 5000);
}
