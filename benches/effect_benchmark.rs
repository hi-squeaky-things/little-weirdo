use std::{fs, sync::Arc};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use little_weirdo::{effects::{Effect, bitcrunch::{Bitcrunch, BitcrunchConfiguration}, delay::{Delay, DelayConfiguration}}, synth::{
    self,
    data::waveforms::{BoxedWaveform, BoxedWaveforms},
}};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

const SAMPLE_RATE: u16 = 44_100; // Audio sample rate in Hz

pub fn criterion_benchmark3(c: &mut Criterion) {

    let mut random = SmallRng::seed_from_u64(0x2238203);

    let bit_crunch_config = BitcrunchConfiguration {
        enabled: true,
    };

    let mut bit_crunch = Bitcrunch::new(bit_crunch_config);

    let delay_config = DelayConfiguration {
        enabled: true,
        delay_time: 500,
        mix_percentage: 80,
        feedback: true,
        feedback_percentage: 70,
        delay_decrease_percentage: 0,
    };

    let mut delay = Delay::new(delay_config, SAMPLE_RATE);


    c.bench_function("Bitcrunch", |b| {
        b.iter_batched(
            || random.random_range((i16::MIN + 1000)..(i16::MAX - 1000)),
            |sample| bit_crunch.clock(sample),
            BatchSize::SmallInput,
        )
    });

     c.bench_function("Delay", |b| {
        b.iter_batched(
            || random.random_range((i16::MIN + 1000)..(i16::MAX - 1000)),
            |sample| delay.clock(sample),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark3);
criterion_main!(benches);