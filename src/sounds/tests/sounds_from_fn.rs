use std::sync::Arc;

use crate::tests::BySample as _;

use crate::{sounds::MemorySound, Sound, Stop};

use super::*;

#[test]
fn basic() {
    let generator = || {
        let sound = MemorySound::from_samples(Arc::new(vec![1, 2]), 2, 1000);
        let sound: Box<dyn Sound> = Box::new(sound);
        Some(sound)
    };
    let mut from_fn = SoundsFromFn::new(Box::new(generator)).by_sample();
    assert_eq!(from_fn.channel_count(), 2);
    assert_eq!(from_fn.sample_rate(), 1000);
    assert_eq!(from_fn.next_frame().unwrap(), vec![1, 2]);
    assert_eq!(from_fn.next_frame().unwrap(), vec![1, 2]);
    // Fills across generated sounds in a single call
    let mut buf = [0; 6];
    let filled = from_fn.next_samples(&mut buf);
    assert!(filled.stop.is_none());
    assert_eq!(buf, [1, 2, 1, 2, 1, 2]);
}

#[test]
fn changing_metadata_and_finishing() {
    let mut num = 0;
    let generator = move || {
        num += 1;
        if num == 3 {
            return None;
        } else if num > 3 {
            unreachable!("should not have been called again");
        }
        let sound = MemorySound::from_samples(Arc::new(vec![1, 2]), 2, 1000 + num);
        let sound: Box<dyn Sound> = Box::new(sound);
        Some(sound)
    };
    let mut from_fn = SoundsFromFn::new(Box::new(generator)).by_sample();
    assert_eq!(from_fn.channel_count(), 2);
    assert_eq!(from_fn.sample_rate(), 1001);
    let mut buf = [0; 6];
    let filled = from_fn.next_samples(&mut buf);
    assert_eq!(filled.written, 2);
    assert!(matches!(filled.stop, Some(Stop::MetadataChanged)));
    assert_eq!(&buf[..2], &[1, 2]);
    assert_eq!(from_fn.sample_rate(), 1002);
    let filled = from_fn.next_samples(&mut buf);
    assert_eq!(filled.written, 2);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert!(matches!(from_fn.next_frame(), Err(Stop::Finished)));
}

#[test]
fn empty_sounds_finish() {
    let generator = || {
        let sound: Box<dyn Sound> = Box::new(crate::sounds::Empty::new(1, 1000));
        Some(sound)
    };
    let mut from_fn = SoundsFromFn::new(Box::new(generator));
    let mut buf = [0; 6];
    let filled = from_fn.next_samples(&mut buf);
    assert_eq!(filled.written, 0);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    let filled = from_fn.next_samples(&mut buf);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
}

#[test]
fn some_empty_sounds_are_skipped() {
    let mut num = 0;
    let generator = move || {
        num += 1;
        let samples = if num % 4 == 0 { vec![1, 2] } else { vec![] };
        let sound: Box<dyn Sound> = Box::new(MemorySound::from_samples(Arc::new(samples), 2, 1000));
        Some(sound)
    };
    let mut from_fn = SoundsFromFn::new(Box::new(generator));
    let mut buf = [0; 6];
    let filled = from_fn.next_samples(&mut buf);
    assert!(filled.stop.is_none());
    assert_eq!(buf, [1, 2, 1, 2, 1, 2]);
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || {
            let mut len = 0;
            let generator = move || {
                len = (len * 7 + 3) % 200;
                let samples: Vec<i16> = (0..len).collect();
                let sound: Box<dyn Sound> =
                    Box::new(MemorySound::from_samples(Arc::new(samples), 1, 1000));
                Some(sound)
            };
            Box::new(SoundsFromFn::new(Box::new(generator)))
        },
        3000,
    );
}
