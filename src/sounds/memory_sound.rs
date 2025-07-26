use std::sync::Arc;

use crate::{NextSample, Sound};

/// A Sound that stores all samples on the heap.
///
/// The heap samples can be shared between multiple MemorySounds that can be
/// played simultaneously. Optionally the sound can repeat forever.
#[derive(Clone)]
pub struct MemorySound {
    samples: Arc<Vec<i16>>,
    channel_count: u16,
    sample_rate: u32,

    next_sample: usize,
    should_loop: bool,
}

/// A [MetadataChanged][NextSample::MetadataChanged] was returned while reading
/// into a [MemorySound] which is not currently supported.
#[derive(Debug)]
pub struct UnsupportedMetadataChangeError {}

impl std::fmt::Display for UnsupportedMetadataChangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unsupported MetadataChanged encountered when consuming Sound"
        )
    }
}

impl std::error::Error for UnsupportedMetadataChangeError {}

impl MemorySound {
    /// Create a MemorySound be consuming another Sound and storing the samples
    /// until it returns `Finished` or `Paused`.
    ///
    /// If an Error is encountered it is returned and any already obtained
    /// samples are lost.
    ///
    /// It is not currently supported for the the originating sample to change
    /// its metadata (i.e. channel count or sample rate). If it does an
    /// IoError of ErrorKind::Other with a UnsupportedMetadataChangeError is
    /// returned.
    pub fn from_sound(mut orig: impl Sound) -> Result<Self, crate::Error> {
        let channel_count = orig.channel_count();
        let sample_rate = orig.sample_rate();

        let mut samples = Vec::new();

        loop {
            let sample = orig.next_sample()?;
            match sample {
                crate::NextSample::Sample(s) => {
                    samples.push(s);
                }
                crate::NextSample::MetadataChanged => {
                    if orig.channel_count() != channel_count || orig.sample_rate() != sample_rate {
                        return Err(crate::Error::IoError(std::io::Error::other(
                            UnsupportedMetadataChangeError {},
                        )));
                    }
                    // Sometimes we see a MetadataChanged from a sound just to
                    // ensure that channels stay in sync. Lets ensure that here
                    // by ensuring that the next sample after MetadataChanged is
                    // for the first channel.
                    let channel_idx = samples.len() % channel_count as usize;
                    if channel_idx != 0 {
                        let outputs_to_stay_in_sync = channel_count as usize - channel_idx;
                        // This should be rare so lets just output 0 for the filler samples.
                        samples.extend(std::iter::repeat(0).take(outputs_to_stay_in_sync));
                    }
                }
                crate::NextSample::Paused | crate::NextSample::Finished => break,
            }
        }

        Ok(MemorySound {
            samples: Arc::new(samples),
            channel_count,
            sample_rate,
            next_sample: 0,
            should_loop: false,
        })
    }

    /// Create memory sound from the raw data of samples.
    ///
    /// Samples should be in the same order as they will be returned from the
    /// next_samples function (e.g. interleaved by channel).
    pub fn from_samples(
        samples: Arc<Vec<i16>>,
        channel_count: u16,
        sample_rate: u32,
    ) -> MemorySound {
        MemorySound {
            samples,
            channel_count,
            sample_rate,
            next_sample: 0,
            should_loop: false,
        }
    }

    /// Instead of finishing after playing all samples, start back at the
    /// beginning and continue forever.
    pub fn set_looping(&mut self, should_loop: bool) {
        self.should_loop = should_loop;
    }
}

impl Sound for MemorySound {
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_sample(&mut self) -> Result<NextSample, crate::Error> {
        if let Some(sample) = self.samples.get(self.next_sample) {
            self.next_sample += 1;
            Ok(NextSample::Sample(*sample))
        } else if self.should_loop && !self.samples.is_empty() {
            self.next_sample = 0;
            self.next_sample()
        } else {
            Ok(NextSample::Finished)
        }
    }

    fn on_start_of_batch(&mut self) {}
}

impl AsRef<[i16]> for MemorySound {
    fn as_ref(&self) -> &[i16] {
        &self.samples
    }
}

#[cfg(test)]
#[path = "./tests/memory_sound.rs"]
mod tests;
