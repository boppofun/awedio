use std::sync::Arc;

use crate::tests::BySample as _;

use crate::{sounds::MemorySound, Sound, Stop};

use super::*;

#[test]
fn empty_gives_metadata_changed_on_next() {
    let mut list = SoundList::new().by_sample();
    assert!(matches!(list.next_frame(), Err(Stop::Finished)));

    let first = MemorySound::from_samples(Arc::new(vec![1, 2, 3, 4]), 4, 1000);
    list.add(Box::new(first));
    let second = MemorySound::from_samples(Arc::new(vec![5, 6]), 2, 8000);
    list.add(Box::new(second));
    assert!(matches!(list.next_frame(), Err(Stop::MetadataChanged)));
    assert_eq!(list.channel_count(), 4);
    assert_eq!(list.sample_rate(), 1000);
    assert_eq!(list.next_frame().unwrap(), vec![1, 2, 3, 4]);
    assert!(matches!(list.next_frame(), Err(Stop::MetadataChanged)));
    assert_eq!(list.channel_count(), 2);
    assert_eq!(list.sample_rate(), 8000);
    assert_eq!(list.next_frame().unwrap(), vec![5, 6]);
    assert!(matches!(list.next_frame(), Err(Stop::Finished)));
}

#[test]
fn fills_across_sounds_with_same_metadata() {
    let mut list = SoundList::new().by_sample();
    list.add(Box::new(MemorySound::from_samples(
        Arc::new(vec![1, 2, 3]),
        1,
        1000,
    )));
    list.add(Box::new(MemorySound::from_samples(
        Arc::new(vec![4, 5]),
        1,
        1000,
    )));
    list.add(Box::new(MemorySound::from_samples(
        Arc::new(vec![6]),
        1,
        2000,
    )));
    assert!(matches!(list.next_sample(), Err(Stop::MetadataChanged)));
    let mut buf = [0; 10];
    let filled = list.next_samples(&mut buf);
    assert_eq!(filled.written, 5);
    assert!(matches!(filled.stop, Some(Stop::MetadataChanged)));
    assert_eq!(&buf[..5], &[1, 2, 3, 4, 5]);
    assert_eq!(list.sample_rate(), 2000);
    let filled = list.next_samples(&mut buf);
    assert_eq!(filled.written, 1);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert_eq!(buf[0], 6);
}

/// Returns `remaining` samples of value 7 and then an error.
struct ErrorAfter {
    remaining: usize,
}

impl Sound for ErrorAfter {
    fn channel_count(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        1000
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> crate::Filled {
        let n = buf.len().min(self.remaining);
        buf[..n].fill(7);
        self.remaining -= n;
        if self.remaining == 0 {
            let error = crate::Error::IoError(std::io::Error::other("test error"));
            crate::Filled::stopped(n, Stop::Error(error))
        } else {
            crate::Filled::all(n)
        }
    }

    fn on_start_of_batch(&mut self) {}
}

#[test]
fn errors_are_skipped() {
    let mut list = SoundList::new();
    list.add(Box::new(MemorySound::from_samples(
        Arc::new(vec![1, 2]),
        1,
        1000,
    )));
    list.add(Box::new(ErrorAfter { remaining: 2 }));
    list.add(Box::new(ErrorAfter { remaining: 0 }));
    list.add(Box::new(MemorySound::from_samples(
        Arc::new(vec![3]),
        1,
        2000,
    )));
    assert!(matches!(
        list.next_samples(&mut []).stop,
        Some(Stop::MetadataChanged)
    ));
    let mut buf = [0; 10];
    let filled = list.next_samples(&mut buf);
    assert_eq!(filled.written, 4);
    assert!(matches!(filled.stop, Some(Stop::MetadataChanged)));
    assert_eq!(&buf[..4], &[1, 2, 7, 7]);
    assert_eq!(list.sample_rate(), 2000);
    let filled = list.next_samples(&mut buf);
    assert_eq!(filled.written, 1);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert_eq!(buf[0], 3);
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || {
            let mut list = SoundList::new().by_sample();
            for len in [3, 100, 1, 257, 50] {
                let samples: Vec<i16> = (0..len * 2).collect();
                list.add(Box::new(MemorySound::from_samples(
                    Arc::new(samples),
                    2,
                    1000,
                )));
            }
            Box::new(list)
        },
        1000,
    );
}
