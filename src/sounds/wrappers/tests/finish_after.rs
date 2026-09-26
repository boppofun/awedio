use super::*;
use crate::tests::BySample as _;
use crate::{sounds::wrappers::SetPaused, tests::ConstantValueSound, Stop};

#[test]
fn test_simple() {
    let mut sound = ConstantValueSound::new(1000)
        .finish_after(Duration::from_millis(100))
        .by_sample();
    for _ in 0..(44100 / 10) {
        assert_eq!(sound.next_sample().unwrap(), 1000);
    }
    assert!(matches!(sound.next_sample(), Err(Stop::Finished)));
}

#[test]
fn test_stops_part_way_through_buffer() {
    let mut sound = ConstantValueSound::new(1000)
        .finish_after(Duration::from_millis(100))
        .by_sample();
    let mut buf = vec![0; 5000];
    let filled = sound.next_samples(&mut buf);
    assert_eq!(filled.written, 4410);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
}

#[test]
fn test_pausing_does_not_count() {
    let mut sound = ConstantValueSound::new(1000)
        .pausable()
        .finish_after(Duration::from_millis(100))
        .by_sample();
    for s in 0..(44100 / 10) {
        if s % 15 == 0 {
            sound.set_paused(true);
            assert!(matches!(sound.next_sample(), Err(Stop::Paused)));
            sound.set_paused(false);
        }
        assert_eq!(sound.next_sample().unwrap(), 1000);
    }
    assert!(matches!(sound.next_sample(), Err(Stop::Finished)));
}

#[test]
fn test_metadata_change_beginning() {
    let mut sound = ConstantValueSound::new(1000)
        .finish_after(Duration::from_millis(100))
        .by_sample();
    sound.inner_mut().inner_mut().set_sample_rate(22050);
    sound.inner_mut().inner_mut().set_channel_count(2);
    assert!(matches!(sound.next_frame(), Err(Stop::MetadataChanged)));
    for _ in 0..(22050 / 10) {
        assert_eq!(sound.next_frame().unwrap(), vec![1000, 1000]);
    }
    assert!(matches!(sound.next_frame(), Err(Stop::Finished)));
}

#[test]
fn test_metadata_change_halfway() {
    let mut sound = ConstantValueSound::new(1000)
        .finish_after(Duration::from_millis(100))
        .by_sample();
    for _ in 0..(44100 / 20) {
        assert_eq!(sound.next_sample().unwrap(), 1000);
    }
    sound.inner_mut().inner_mut().set_sample_rate(88200);
    sound.inner_mut().inner_mut().set_channel_count(4);
    assert!(matches!(sound.next_frame(), Err(Stop::MetadataChanged)));
    for _ in 0..(88200 / 20) {
        assert_eq!(sound.next_frame().unwrap(), vec![1000; 4]);
    }
    assert!(matches!(sound.next_frame(), Err(Stop::Finished)));
}

#[test]
fn test_metadata_change_end() {
    let mut sound = ConstantValueSound::new(1000)
        .finish_after(Duration::from_millis(100))
        .by_sample();
    for _ in 0..(44100 / 10) {
        assert_eq!(sound.next_sample().unwrap(), 1000);
    }
    assert!(matches!(sound.next_sample(), Err(Stop::Finished)));
    sound.inner_mut().inner_mut().set_sample_rate(22050);
    sound.inner_mut().inner_mut().set_channel_count(2);
    assert!(matches!(sound.next_frame(), Err(Stop::Finished)));
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || Box::new(crate::tests::Sawtooth::new(2, 44100).finish_after(Duration::from_millis(50))),
        5000,
    );
}
