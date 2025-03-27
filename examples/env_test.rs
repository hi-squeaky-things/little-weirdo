use little_weirdo::synth::{envelope::{EnvelopConfiguration, EnvelopeGenerator}, Clockable};

fn main() {
   println!("test envelope");
   let config = EnvelopConfiguration {
    attack_time: 5,
    decay_time: 100,
    release_time: 100,
    sustain_level: 80,
   };

   let mut generator = EnvelopeGenerator::new(config, 44_100);

    generator.open_gate();
    for i in 0..300 {
        let value = generator.clock(None);
        println!("{:?} :: value = {:?}", i, value);
    }

}