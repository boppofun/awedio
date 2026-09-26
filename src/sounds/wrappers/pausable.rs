use crate::{sounds::wrappers::SetStopped, Sound};

use super::{SetSpeed, SetVolume};

/// A Sound which can be paused.
pub trait SetPaused {
    /// Pause or unpause the sound.
    fn set_paused(&mut self, paused: bool);
}

/// A wrapper to make a Sound pausable.
pub struct Pausable<S: Sound> {
    inner: S,
    paused: bool,
}

impl<S> Pausable<S>
where
    S: Sound,
{
    /// Wrap `inner` and allow it to be paused via
    /// [set_paused][SetPaused::set_paused].
    pub fn new(inner: S) -> Self {
        Pausable {
            inner,
            paused: false,
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

impl<S> Sound for Pausable<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    #[inline]
    fn next_samples(&mut self, buf: &mut [i16]) -> crate::Filled {
        if self.paused {
            return crate::Filled::stopped(0, crate::Stop::Paused);
        }
        self.inner.next_samples(buf)
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S> Pausable<S>
where
    S: Sound,
{
    /// Return if the Sound is currently being paused.
    pub fn paused(&self) -> bool {
        self.paused
    }
}

impl<S> SetPaused for Pausable<S>
where
    S: Sound,
{
    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
}

impl<S> SetStopped for Pausable<S>
where
    S: Sound + SetStopped,
{
    fn set_stopped(&mut self) {
        self.inner.set_stopped()
    }
}

impl<S> SetVolume for Pausable<S>
where
    S: Sound + SetVolume,
{
    fn set_volume(&mut self, multiplier: f32) {
        self.inner.set_volume(multiplier)
    }
}

impl<S> SetSpeed for Pausable<S>
where
    S: Sound + SetSpeed,
{
    fn set_speed(&mut self, multiplier: f32) {
        self.inner.set_speed(multiplier)
    }
}

#[cfg(test)]
#[path = "./tests/pausable.rs"]
mod tests;
