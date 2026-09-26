//! Common utilities for tests.
//!
//! Note that other files under the tests/ folders are not submodules of this
//! mod but are submodules of the modules they are testing.

use crate::{Filled, Sound, Stop};

pub const DEFAULT_SAMPLE_RATE: u32 = 44100;
pub const DEFAULT_CHANNEL_COUNT: u16 = 1;

/// Only useful for tests as a constant offset makes no hearable sound.
pub struct ConstantValueSound {
    pub value: i16,
    pub channel_count: u16,
    pub sample_rate: u32,
    pub metadata_changed: bool,
}

impl ConstantValueSound {
    pub fn new(value: i16) -> ConstantValueSound {
        ConstantValueSound {
            value,
            channel_count: DEFAULT_CHANNEL_COUNT,
            sample_rate: DEFAULT_SAMPLE_RATE,
            metadata_changed: false,
        }
    }
}

impl Sound for ConstantValueSound {
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        if self.metadata_changed {
            self.metadata_changed = false;
            return Filled::stopped(0, Stop::MetadataChanged);
        }
        buf.fill(self.value);
        Filled::all(buf.len())
    }

    fn on_start_of_batch(&mut self) {}
}

impl ConstantValueSound {
    pub fn set_channel_count(&mut self, new_count: u16) {
        self.channel_count = new_count;
        self.metadata_changed = true;
    }

    pub fn set_sample_rate(&mut self, new_rate: u32) {
        self.sample_rate = new_rate;
        self.metadata_changed = true;
    }
}

/// Start at 0, increment by 1 until MAX value then jump to MIN value and
/// increment by 1 again
pub struct Sawtooth {
    pub value: i16,
    pub channel_count: u16,
    pub channel_idx: u16,
    pub sample_rate: u32,
}

impl Sawtooth {
    pub fn new(channel_count: u16, sample_rate: u32) -> Sawtooth {
        Sawtooth {
            value: 0,
            channel_count,
            channel_idx: 0,
            sample_rate,
        }
    }
}

impl Sound for Sawtooth {
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        for sample in buf.iter_mut() {
            *sample = self.value;
            self.channel_idx += 1;
            if self.channel_idx == self.channel_count {
                self.channel_idx = 0;
                self.value = self.value.wrapping_add(1);
            }
        }
        Filled::all(buf.len())
    }

    fn on_start_of_batch(&mut self) {}
}

/// Pull `num_frames` frames from `sound` in calls of `frames_per_call` frames
/// (restarting a batch before every call) and return all samples written along
/// with the stops encountered (and the number of samples written before each
/// stop). Stops of `Paused` or `Finished` end the collection. Pulling
/// continues after `MetadataChanged` (the channel count must not change).
pub fn collect(
    sound: &mut dyn Sound,
    num_frames: usize,
    frames_per_call: usize,
) -> (Vec<i16>, Vec<(usize, String)>) {
    let channel_count = sound.channel_count() as usize;
    let mut samples = Vec::new();
    let mut stops = Vec::new();
    let mut buf = vec![0_i16; frames_per_call * channel_count];
    while samples.len() < num_frames * channel_count {
        let remaining = num_frames * channel_count - samples.len();
        let len = remaining.min(buf.len());
        sound.on_start_of_batch();
        let filled = sound.next_samples(&mut buf[..len]);
        assert!(filled.written <= len);
        assert_eq!(filled.written % channel_count, 0);
        samples.extend_from_slice(&buf[..filled.written]);
        match filled.stop {
            None => assert_eq!(filled.written, len),
            Some(stop) => {
                let done = !matches!(stop, Stop::MetadataChanged);
                stops.push((samples.len(), format!("{stop:?}")));
                assert_eq!(sound.channel_count() as usize, channel_count);
                if done {
                    break;
                }
            }
        }
    }
    (samples, stops)
}

/// Assert that pulling from sounds created by `make_sound` gives the same
/// samples and stops regardless of how many frames are requested at a time.
pub fn assert_fill_size_independent(
    mut make_sound: impl FnMut() -> Box<dyn Sound>,
    num_frames: usize,
) {
    let (reference, reference_stops) = collect(make_sound().as_mut(), num_frames, 1);
    assert!(!reference.is_empty());
    for frames_per_call in [7, 128, 1000] {
        let (samples, stops) = collect(make_sound().as_mut(), num_frames, frames_per_call);
        assert_eq!(
            samples, reference,
            "samples differ with {frames_per_call} frames per call"
        );
        // Where a MetadataChanged lands can depend on the fill size (e.g. a
        // sound can return it with 0 samples written) but Paused/Finished
        // must be the same.
        let final_stop = |stops: &[(usize, String)]| {
            stops
                .iter()
                .filter(|(_, s)| s != "MetadataChanged")
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            final_stop(&stops),
            final_stop(&reference_stops),
            "stops differ with {frames_per_call} frames per call"
        );
    }
}

/// Wrap a sound in a [SampleBySample] to pull a sample or frame at a time.
pub trait BySample: Sound + Sized {
    fn by_sample(self) -> crate::sounds::wrappers::SampleBySample<Self> {
        crate::sounds::wrappers::SampleBySample::new(self)
    }
}

impl<S: Sound> BySample for S {}
