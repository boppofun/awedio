use std::sync::Arc;

use crate::{Filled, Sound, Stop};

/// A Sound that stores all samples on the heap.
///
/// The heap samples can be shared between multiple MemorySounds that can be
/// played simultaneously. Optionally the sound can repeat forever.
#[derive(Clone)]
pub struct MemorySound {
    samples: Arc<Vec<i16>>,
    /// Number of samples in `samples` that are part of complete frames.
    len: usize,
    channel_count: u16,
    sample_rate: u32,

    next_sample: usize,
    should_loop: bool,
}

/// A [MetadataChanged][Stop::MetadataChanged] was returned while reading
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

        let mut samples: Vec<i16> = Vec::new();
        let chunk_len = 1024 * channel_count as usize;

        loop {
            let start = samples.len();
            samples.resize(start + chunk_len, 0);
            let filled = orig.next_samples(&mut samples[start..]);
            samples.truncate(start + filled.written);
            match filled.stop {
                None => (),
                Some(Stop::MetadataChanged) => {
                    if orig.channel_count() != channel_count || orig.sample_rate() != sample_rate {
                        return Err(crate::Error::IoError(std::io::Error::other(
                            UnsupportedMetadataChangeError {},
                        )));
                    }
                }
                Some(Stop::Paused) | Some(Stop::Finished) => break,
                Some(Stop::Error(e)) => return Err(e),
            }
        }
        samples.shrink_to_fit();

        Ok(MemorySound {
            len: samples.len(),
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
    /// next_samples function (e.g. interleaved by channel). If the number of
    /// samples is not a multiple of `channel_count`, the samples of the last
    /// incomplete frame are ignored.
    pub fn from_samples(
        samples: Arc<Vec<i16>>,
        channel_count: u16,
        sample_rate: u32,
    ) -> MemorySound {
        let len = samples.len() - samples.len() % channel_count as usize;
        MemorySound {
            samples,
            len,
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

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let samples = &self.samples[..self.len];
        let mut written = 0;
        while written < buf.len() {
            if self.next_sample >= samples.len() {
                if self.should_loop && !samples.is_empty() {
                    self.next_sample = 0;
                } else {
                    return Filled::stopped(written, Stop::Finished);
                }
            }
            let to_copy = (buf.len() - written).min(samples.len() - self.next_sample);
            buf[written..written + to_copy]
                .copy_from_slice(&samples[self.next_sample..self.next_sample + to_copy]);
            written += to_copy;
            self.next_sample += to_copy;
        }
        Filled::all(written)
    }

    fn on_start_of_batch(&mut self) {}
}

impl AsRef<[i16]> for MemorySound {
    fn as_ref(&self) -> &[i16] {
        &self.samples[..self.len]
    }
}

#[cfg(test)]
#[path = "./tests/memory_sound.rs"]
mod tests;
