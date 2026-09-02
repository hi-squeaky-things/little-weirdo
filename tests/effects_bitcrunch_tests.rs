use little_weirdo::effects::{
    bitcrunch::{Bitcrunch, BitcrunchConfiguration},
    Effect,
};

#[test]
fn bitcrunch_applies_quantization_when_enabled() {
    let mut effect = Bitcrunch::new(BitcrunchConfiguration { enabled: true });
    assert_eq!(effect.clock(0), 0);
    assert_eq!(effect.clock(1), 0);
    assert_eq!(effect.clock(255), 0);
    assert_eq!(effect.clock(256), 256);
    assert_eq!(effect.clock(511), 256);
    assert_eq!(effect.clock(-1), 0);
    assert_eq!(effect.clock(-256), -256);
    assert_eq!(effect.clock(-511), -256);
}

#[test]
fn bitcrunch_leaves_signal_unchanged_when_disabled() {
    let mut effect = Bitcrunch::new(BitcrunchConfiguration { enabled: false });
    assert_eq!(effect.clock(1234), 1234);
    assert_eq!(effect.clock(-4321), -4321);
    assert_eq!(effect.clock(32767), 32767);
    assert_eq!(effect.clock(-32768), -32768);
}
