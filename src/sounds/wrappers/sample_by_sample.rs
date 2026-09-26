use crate::{Filled, Sound, Stop};

use super::Wrapper;

/// Pull samples from a Sound one sample or one frame at a time.
///
/// This is intended for tests and other rare callers. It is much less
/// efficient than calling [Sound::next_samples] with a larger buffer.
///
/// A Sound may report a stop together with the samples it wrote. The rest of
/// the frame and the stop are stored and returned by the following calls so
/// nothing is lost.
///
/// SampleBySample is also a Sound itself. Any stored samples and stop are
/// returned from `next_samples` before pulling from the inner Sound again.
pub struct SampleBySample<S: Sound> {
    inner: S,
    /// Samples of the current frame not yet returned.
    pending: Vec<i16>,
    next_pending_idx: usize,
    pending_stop: Option<Stop>,
}

impl<S: Sound> SampleBySample<S> {
    /// Wrap `inner` so it can be pulled one sample at a time.
    pub fn new(inner: S) -> Self {
        SampleBySample {
            inner,
            pending: Vec::new(),
            next_pending_idx: 0,
            pending_stop: None,
        }
    }

    /// Unwrap and return the wrapped Sound. Any stored samples or stop are
    /// lost.
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Retrieve the next sample.
    ///
    /// The first sample is for the first channel, the second for the second
    /// channel and so on. Returns Err if the Sound stopped before another
    /// sample was written.
    pub fn next_sample(&mut self) -> Result<i16, Stop> {
        if let Some(sample) = self.pending.get(self.next_pending_idx) {
            self.next_pending_idx += 1;
            return Ok(*sample);
        }
        if let Some(stop) = self.pending_stop.take() {
            return Err(stop);
        }
        self.fill_frame();
        if let Some(sample) = self.pending.first() {
            self.next_pending_idx = 1;
            return Ok(*sample);
        }
        Err(self.pending_stop.take().unwrap_or(Stop::Finished))
    }

    /// Retrieve the next frame (one sample per channel).
    ///
    /// Returns Err if the Sound stopped before a frame was written.
    ///
    /// # Panics
    ///
    /// Panics if called part way through a frame (i.e. after `next_sample`
    /// returned some but not all samples of a frame).
    pub fn next_frame(&mut self) -> Result<Vec<i16>, Stop> {
        assert_eq!(
            self.next_pending_idx,
            self.pending.len(),
            "next_frame called part way through a frame"
        );
        if let Some(stop) = self.pending_stop.take() {
            return Err(stop);
        }
        self.fill_frame();
        if self.pending.is_empty() {
            return Err(self.pending_stop.take().unwrap_or(Stop::Finished));
        }
        self.next_pending_idx = self.pending.len();
        Ok(self.pending.clone())
    }

    /// Pull one frame from inner into `pending` and store any stop.
    fn fill_frame(&mut self) {
        self.pending.clear();
        self.pending.resize(self.inner.channel_count() as usize, 0);
        let filled = self.inner.next_samples(&mut self.pending);
        self.pending.truncate(filled.written);
        self.next_pending_idx = 0;
        self.pending_stop = filled.stop;
        if self.pending.is_empty() && self.pending_stop.is_none() {
            // Only possible for a sound with 0 channels
            self.pending_stop = Some(Stop::Finished);
        }
    }
}

impl<S: Sound> Sound for SampleBySample<S> {
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let pending = &self.pending[self.next_pending_idx..];
        let written = pending.len().min(buf.len());
        buf[..written].copy_from_slice(&pending[..written]);
        self.next_pending_idx += written;
        if self.next_pending_idx < self.pending.len() {
            return Filled::all(written);
        }
        if let Some(stop) = self.pending_stop.take() {
            return Filled::stopped(written, stop);
        }
        let filled = self.inner.next_samples(&mut buf[written..]);
        Filled {
            written: written + filled.written,
            stop: filled.stop,
        }
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S: Sound> Wrapper for SampleBySample<S> {
    type Inner = S;

    fn inner(&self) -> &S {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    fn into_inner(self) -> S {
        self.inner
    }
}

#[cfg(test)]
#[path = "./tests/sample_by_sample.rs"]
mod tests;
