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
