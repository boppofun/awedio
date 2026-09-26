use super::*;
use crate::tests::BySample as _;
use crate::tests::ConstantValueSound;

#[test]
fn adjust_down() {
    let mut first = ConstantValueSound::new(1000)
        .with_adjustable_volume()
        .by_sample();
    first.set_volume(0.5);
    assert_eq!(first.next_sample().unwrap(), 500);
}

#[test]
fn adjust_up() {
    let mut first = ConstantValueSound::new(1000)
        .with_adjustable_volume()
        .by_sample();
    first.set_volume(5.0);
    assert_eq!(first.next_sample().unwrap(), 5000);
}

#[test]
fn test_saturation() {
    let mut first = ConstantValueSound::new(1000)
        .with_adjustable_volume()
        .by_sample();
    first.set_volume(1000.0);
    assert_eq!(first.next_sample().unwrap(), i16::MAX);
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || Box::new(crate::tests::Sawtooth::new(2, 1000).with_adjustable_volume_of(0.7)),
        3000,
    );
}
