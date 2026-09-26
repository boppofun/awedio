use std::time::Duration;

use crate::Sound;

use super::Wrapper;

/// Play the  first part of an inner Sound measured in seconds then stop even
/// if the inner sound still has samples remaining. Finishes early if the inner
/// sound finishes before duration. Any time the inner sample is paused does not
/// count against the duration (i.e. duration only includes time of samples).
pub struct FinishAfter<S: Sound> {
    inner: S,
    frames_remaining: u64,
    total_duration: Duration,
    current_channel_count: u16,
    current_sample_rate: u32,
}

impl<S> FinishAfter<S>
where
    S: Sound,
{
    /// Only play the first `duration` of inner before finishing.
    pub fn new(inner: S, duration: Duration) -> Self {
        let current_channel_count = inner.channel_count();
        let current_sample_rate = inner.sample_rate();
        let frames_remaining = num_frames(duration, current_sample_rate);
        FinishAfter {
            inner,
            frames_remaining,
            total_duration: duration,
            current_channel_count,
            current_sample_rate,
        }
    }

    /// Get a reference to the wrapped inner Sound.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get a mutable reference to the wrapped inner Sound.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Unwrap and return the previously wrapped Sound.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S> Sound for FinishAfter<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> crate::Filled {
        if self.frames_remaining == 0 {
            return crate::Filled::stopped(0, crate::Stop::Finished);
        }
        let channel_count = self.current_channel_count as usize;
        let max_len = self
            .frames_remaining
            .saturating_mul(channel_count as u64)
            .min(buf.len() as u64) as usize;
        let filled = self.inner.next_samples(&mut buf[..max_len]);
        self.frames_remaining -= (filled.written / channel_count) as u64;
        match &filled.stop {
            None => {
                if max_len < buf.len() {
                    // We hit our duration limit before filling buf.
                    debug_assert_eq!(self.frames_remaining, 0);
                    return crate::Filled::stopped(filled.written, crate::Stop::Finished);
                }
            }
            Some(crate::Stop::MetadataChanged) => {
                let total_old_frames = num_frames(self.total_duration, self.current_sample_rate);
                let num_frames_played = total_old_frames - self.frames_remaining;
                let seconds_played = num_frames_played as f64 / self.current_sample_rate as f64;
                let duration_played = Duration::from_secs_f64(seconds_played);
                let duration_remaining = self.total_duration.saturating_sub(duration_played);
                self.current_channel_count = self.inner.channel_count();
                self.current_sample_rate = self.inner.sample_rate();
                self.frames_remaining = num_frames(duration_remaining, self.current_sample_rate);
            }
            Some(_) => (),
        }
        filled
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

fn num_frames(duration: Duration, sample_rate: u32) -> u64 {
    const MICROS_PER_SEC: u64 = 1_000_000;
    let micros = duration.as_secs() * MICROS_PER_SEC + duration.subsec_micros() as u64;
    micros * sample_rate as u64 / MICROS_PER_SEC
}

impl<S: Sound> Wrapper for FinishAfter<S> {
    type Inner = S;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }

    fn into_inner(self) -> Self::Inner {
        self.inner
    }
}

#[cfg(test)]
#[path = "./tests/finish_after.rs"]
mod tests;
