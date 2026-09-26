use crate::tests::BySample as _;
use crate::tests::Sawtooth;

use super::*;

#[test]
fn mono_to_stereo() {
    let mut sound = ChannelCountConverter::new(Sawtooth::new(1, 1000), 2).by_sample();
    assert_eq!(sound.channel_count(), 2);
    let mut buf = [0; 6];
    assert!(sound.next_samples(&mut buf).stop.is_none());
    assert_eq!(buf, [0, 0, 1, 1, 2, 2]);
}

#[test]
fn stereo_to_mono() {
    let samples = vec![0, 10, 100, 200, -5, -6];
    let inner = crate::sounds::MemorySound::from_samples(std::sync::Arc::new(samples), 2, 1000);
    let mut sound = ChannelCountConverter::new(inner, 1).by_sample();
    assert_eq!(sound.channel_count(), 1);
    let mut buf = [0; 4];
    let filled = sound.next_samples(&mut buf);
    assert_eq!(filled.written, 3);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert_eq!(&buf[..3], &[5, 150, -5]);
}

#[test]
fn channel_count_change_of_inner() {
    let mut inner = crate::tests::ConstantValueSound::new(7);
    inner.set_channel_count(2);
    // Clear the pending MetadataChanged
    let _ = inner.next_samples(&mut []);
    let mut sound = ChannelCountConverter::new(inner, 2).by_sample();
    assert_eq!(sound.next_frame().unwrap(), vec![7, 7]);
    sound.inner_mut().inner_mut().set_channel_count(1);
    assert!(matches!(sound.next_frame(), Err(Stop::MetadataChanged)));
    assert_eq!(sound.next_frame().unwrap(), vec![7, 7]);
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || Box::new(ChannelCountConverter::new(Sawtooth::new(1, 1000), 2)),
        3000,
    );
    crate::tests::assert_fill_size_independent(
        || Box::new(ChannelCountConverter::new(Sawtooth::new(2, 1000), 1)),
        3000,
    );
}
