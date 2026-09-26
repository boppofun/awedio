use crate::sounds::wrappers::Wrapper as _;
use crate::tests::BySample as _;
use crate::{sounds::SoundList, Sound, Stop};

use super::*;

#[test]
fn from_samples_ignores_partial_frame() {
    let sound = MemorySound::from_samples(Arc::new(vec![1, 2, 3, 4, 1, 2]), 4, 1000);
    assert_eq!(sound.as_ref(), &[1, 2, 3, 4]);
}

#[test]
fn from_sound_list() {
    let first = MemorySound::from_samples(Arc::new(vec![1, 2, 3, 4]), 4, 1000);
    let second = MemorySound::from_samples(Arc::new(vec![5, 6, 7, 8]), 4, 1000);
    let mut list = SoundList::new();
    list.add(Box::new(first));
    list.add(Box::new(second));
    let mut combined = list.into_memory_sound().unwrap().by_sample();
    assert_eq!(combined.sample_rate(), 1000);
    assert_eq!(combined.channel_count(), 4);
    assert_eq!(combined.inner().as_ref(), &[1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(combined.next_frame().unwrap(), vec![1, 2, 3, 4]);
    assert_eq!(combined.next_frame().unwrap(), vec![5, 6, 7, 8]);
    assert!(matches!(combined.next_frame(), Err(Stop::Finished)));
}

#[test]
fn from_sound_with_different_metadata_errors() {
    let first = MemorySound::from_samples(Arc::new(vec![1, 2, 3, 4]), 4, 1000);
    let second = MemorySound::from_samples(Arc::new(vec![5, 6]), 2, 1000);
    let mut list = SoundList::new();
    list.add(Box::new(first));
    list.add(Box::new(second));
    assert!(list.into_memory_sound().is_err());
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || {
            let samples: Vec<i16> = (0..2001).collect();
            Box::new(MemorySound::from_samples(Arc::new(samples), 1, 1000))
        },
        3000,
    );
    crate::tests::assert_fill_size_independent(
        || {
            let samples: Vec<i16> = (0..2000).collect();
            let mut sound = MemorySound::from_samples(Arc::new(samples), 2, 1000);
            sound.set_looping(true);
            Box::new(sound)
        },
        3000,
    );
}

#[test]
fn loop_forever() {
    let mut sound = MemorySound::from_samples(Arc::new(vec![1, 2]), 1, 1000);
    sound.set_looping(true);
    let mut sound = sound.by_sample();
    assert_eq!(sound.next_sample().unwrap(), 1);
    assert_eq!(sound.next_sample().unwrap(), 2);
    assert_eq!(sound.next_sample().unwrap(), 1);
    assert_eq!(sound.next_sample().unwrap(), 2);
    assert_eq!(sound.next_sample().unwrap(), 1);
    let mut buf = [0; 5];
    let filled = sound.next_samples(&mut buf);
    assert!(filled.stop.is_none());
    assert_eq!(buf, [2, 1, 2, 1, 2]);
}
