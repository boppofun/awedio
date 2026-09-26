use std::sync::Arc;

use super::*;
use crate::sounds::MemorySound;
use crate::Stop;

/// Writes all remaining samples and reports Finished in the same call, even
/// when the buffer is full.
struct EagerStop {
    samples: Vec<i16>,
    channel_count: u16,
}

impl Sound for EagerStop {
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        1000
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let n = buf.len().min(self.samples.len());
        buf[..n].copy_from_slice(&self.samples[..n]);
        self.samples.drain(..n);
        if self.samples.is_empty() {
            Filled::stopped(n, Stop::MetadataChanged)
        } else {
            Filled::all(n)
        }
    }

    fn on_start_of_batch(&mut self) {}
}

#[test]
fn stop_with_full_buffer_is_not_lost() {
    let mut sound = SampleBySample::new(EagerStop {
        samples: vec![1, 2],
        channel_count: 1,
    });
    assert_eq!(sound.next_sample().unwrap(), 1);
    assert_eq!(sound.next_sample().unwrap(), 2);
    assert!(matches!(sound.next_sample(), Err(Stop::MetadataChanged)));
}

#[test]
fn stereo_samples() {
    let mut sound = SampleBySample::new(EagerStop {
        samples: vec![1, 2, 3, 4],
        channel_count: 2,
    });
    assert_eq!(sound.next_sample().unwrap(), 1);
    assert_eq!(sound.next_sample().unwrap(), 2);
    assert_eq!(sound.next_frame().unwrap(), vec![3, 4]);
    assert!(matches!(sound.next_frame(), Err(Stop::MetadataChanged)));
}

#[test]
fn next_samples_returns_stored_samples_first() {
    let inner = MemorySound::from_samples(Arc::new(vec![1, 2, 3, 4, 5, 6]), 2, 1000);
    let mut sound = SampleBySample::new(inner);
    assert_eq!(sound.next_sample().unwrap(), 1);
    let mut buf = [0; 1];
    assert!(sound.next_samples(&mut buf).stop.is_none());
    assert_eq!(buf, [2]);
    let mut buf = [0; 6];
    let filled = sound.next_samples(&mut buf);
    assert_eq!(filled.written, 4);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert_eq!(&buf[..4], &[3, 4, 5, 6]);
}
