use crate::Sound;

use super::{SetSpeed, SetVolume};

/// A Sound which can be stopped.
pub trait SetStopped {
    /// Stop the sound
    fn set_stopped(&mut self);
}

/// A wrapper to make a Sound stoppable.
/// A stopped sound will always return `NextSample::Finished`.
pub struct Stoppable<S: Sound> {
    inner: S,
    stopped: bool,
}

impl<S> Stoppable<S>
where
    S: Sound,
{
    /// Wrap `inner` and allow it to be stopped via
    /// [set_stopped][SetStopped::set_stopped].
    pub fn new(inner: S) -> Self {
        Stoppable {
            inner,
            stopped: false,
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

impl<S> Sound for Stoppable<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn next_sample(&mut self) -> Result<crate::NextSample, crate::Error> {
        if self.stopped {
            return Ok(crate::NextSample::Finished);
        }
        self.inner.next_sample()
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S> Stoppable<S>
where
    S: Sound,
{
    /// Return if the Sound is stopped.
    pub fn stopped(&self) -> bool {
        self.stopped
    }
}

impl<S> SetStopped for Stoppable<S>
where
    S: Sound,
{
    fn set_stopped(&mut self) {
        self.stopped = true;
    }
}

impl<S> SetVolume for Stoppable<S>
where
    S: Sound + SetVolume,
{
    fn set_volume(&mut self, multiplier: f32) {
        self.inner.set_volume(multiplier)
    }
}

impl<S> SetSpeed for Stoppable<S>
where
    S: Sound + SetSpeed,
{
    fn set_speed(&mut self, multiplier: f32) {
        self.inner.set_speed(multiplier)
    }
}

#[cfg(test)]
#[path = "./tests/stoppable.rs"]
mod tests;
