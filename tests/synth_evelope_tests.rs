use little_weirdo::synth::{
    envelope::{EnvelopConfiguration, EnvelopeGenerator},
    Clockable,
};

#[test]
fn envelope_attack_time_1ms_reaches_full_level() {
    let mut env = EnvelopeGenerator::new(
        EnvelopConfiguration {
            attack_time: 1,
            decay_time: 10,
            release_time: 10,
            sustain_level: 50,
        },
        1000,
    );

    env.open_gate();
    let mut value = 0;
    for _ in 0..2 {
        value = env.clock(None);
    }
    assert_eq!(value, 100);
}

#[test]
fn envelope_attack_time_5000ms_reaches_full_level() {
    let mut env = EnvelopeGenerator::new(
        EnvelopConfiguration {
            attack_time: 5000,
            decay_time: 10,
            release_time: 10,
            sustain_level: 50,
        },
        1000,
    );

    env.open_gate();

    let mut last_value = 0;
    for _ in 0..5001 {
        last_value = env.clock(None);
    }

    assert_eq!(last_value, 100);
}

#[test]
fn updating_sustain_preserves_the_active_envelope() {
    let mut env = EnvelopeGenerator::new(
        EnvelopConfiguration {
            attack_time: 1,
            decay_time: 1,
            release_time: 10,
            sustain_level: 50,
        },
        1000,
    );

    env.open_gate();
    for _ in 0..3 {
        env.clock(None);
    }
    assert_eq!(env.clock(None), 50);

    env.update_configuration(EnvelopConfiguration {
        attack_time: 1,
        decay_time: 1,
        release_time: 10,
        sustain_level: 75,
    });

    assert_eq!(env.clock(None), 75);
}

#[test]
fn changing_release_time_uses_milliseconds_at_the_sample_rate() {
    let mut env = EnvelopeGenerator::new(
        EnvelopConfiguration {
            attack_time: 1,
            decay_time: 1,
            release_time: 1000,
            sustain_level: 100,
        },
        44_100,
    );

    env.open_gate();
    for _ in 0..100 {
        env.clock(None);
    }

    env.update_configuration(EnvelopConfiguration {
        attack_time: 1,
        decay_time: 1,
        release_time: 5000,
        sustain_level: 100,
    });
    env.close_gate();

    let release_ticks = 5_000 * 44_100 / 1_000;
    for _ in 0..release_ticks {
        assert!(env.clock(None) > 0);
    }
    assert_eq!(env.clock(None), 0);
}

#[test]
fn release_works_when_note_ends_during_a_long_decay() {
    let mut env = EnvelopeGenerator::new(
        EnvelopConfiguration {
            attack_time: 1,
            decay_time: 5000,
            release_time: 1000,
            sustain_level: 20,
        },
        44_100,
    );

    env.open_gate();
    for _ in 0..(1000 * 44_100 / 1000) {
        env.clock(None);
    }

    env.close_gate();
    let release_start = env.clock(None);
    assert!(release_start > 20);

    for _ in 1..(950 * 44_100 / 1000) {
        assert!(env.clock(None) > 0);
    }
    for _ in 0..(50 * 44_100 / 1000) {
        env.clock(None);
    }
    assert_eq!(env.clock(None), 0);
}

#[test]
fn retriggered_attack_rises_from_current_level_for_the_remaining_attack_time() {
    let mut env = EnvelopeGenerator::new(
        EnvelopConfiguration {
            attack_time: 100,
            decay_time: 10,
            release_time: 10,
            sustain_level: 50,
        },
        44_100,
    );

    env.open_gate();
    for _ in 0..1000 {
        env.clock(None);
    }
    let level_at_retrigger = env.clock(None);
    env.open_gate();

    assert_eq!(env.clock(None), level_at_retrigger);

    let remaining_attack_ticks = 4_410 * (100 - level_at_retrigger as u32) / 100;
    let mut level = level_at_retrigger;
    for _ in 1..remaining_attack_ticks {
        level = env.clock(None);
    }
    assert!(level < 100);
    assert_eq!(env.clock(None), 100);
}
