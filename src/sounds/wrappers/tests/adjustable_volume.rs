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
fn fixed_point_matches_float() {
    for volume in [0.001, 0.1, 0.25, 0.3333, 0.5, 0.9, 1.5, 3.0, 15.9] {
        let samples: Vec<i16> = (i16::MIN..=i16::MAX).step_by(7).collect();
        let sound =
            crate::sounds::MemorySound::from_samples(std::sync::Arc::new(samples.clone()), 1, 1000);
        let mut sound = sound.with_adjustable_volume_of(volume).by_sample();
        let mut buf = vec![0; samples.len()];
        assert!(sound.next_samples(&mut buf).stop.is_none());
        for (orig, adjusted) in samples.iter().zip(buf) {
            let expected = (*orig as f32 * volume).clamp(i16::MIN as f32, i16::MAX as f32);
            // Fixed point has 12 fractional bits so the error grows with the
            // sample value.
            let tolerance = 1.0 + orig.unsigned_abs() as f32 / 4096.0;
            assert!(
                (adjusted as f32 - expected).abs() <= tolerance,
                "volume {volume} sample {orig}: got {adjusted} expected {expected}"
            );
        }
    }
}

#[test]
fn mute_and_unity() {
    let mut sound = ConstantValueSound::new(1000)
        .with_adjustable_volume_of(0.0)
        .by_sample();
    assert_eq!(sound.next_sample().unwrap(), 0);
    sound.set_volume(1.0);
    assert_eq!(sound.next_sample().unwrap(), 1000);
    sound.set_volume(-1.0);
    assert_eq!(sound.next_sample().unwrap(), -1000);
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || Box::new(crate::tests::Sawtooth::new(2, 1000).with_adjustable_volume_of(0.7)),
        3000,
    );
}
