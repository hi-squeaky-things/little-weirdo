use little_weirdo::synth::noise::{NoiseGenerator, NoiseKind};

fn samples(kind: NoiseKind, seed: u64) -> [i16; 128] {
    let mut generator = NoiseGenerator::new(kind, seed);
    let mut output = [0; 128];
    for sample in &mut output {
        *sample = generator.sample();
    }
    output
}

#[test]
fn noise_generators_are_bounded_and_nonconstant() {
    for kind in [NoiseKind::White, NoiseKind::Pink, NoiseKind::Brown] {
        let output = samples(kind, 42);
        assert!(output
            .iter()
            .all(|sample| i32::from(*sample) <= i32::from(i16::MAX)));
        assert!(output
            .iter()
            .all(|sample| i32::from(*sample) >= i32::from(i16::MIN)));
        assert!(output.windows(2).any(|window| window[0] != window[1]));
    }
}

#[test]
fn noise_generators_are_deterministic_but_modes_differ() {
    assert_eq!(samples(NoiseKind::White, 42), samples(NoiseKind::White, 42));
    assert_ne!(samples(NoiseKind::White, 42), samples(NoiseKind::Pink, 42));
    assert_ne!(samples(NoiseKind::Pink, 42), samples(NoiseKind::Brown, 42));
}

#[test]
fn brown_noise_stays_bounded_over_time() {
    let output = samples(NoiseKind::Brown, 42);
    let mut generator = NoiseGenerator::new(NoiseKind::Brown, 42);
    for _ in 0..100_000 {
        let sample = generator.sample();
        assert!(sample >= i16::MIN);
        assert!(sample <= i16::MAX);
    }
    assert_ne!(output, [0; 128]);
}
