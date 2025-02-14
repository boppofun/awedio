use crate::Sound;

use super::{SetPaused, SetStopped, SetVolume};

/// A sound that can have the playback speed adjusted.
///
/// This is [pitch control](https://en.wikipedia.org/wiki/Pitch_control) affecting both the pitch and speed and not [time stretching](https://en.wikipedia.org/wiki/Audio_time_stretching_and_pitch_scaling) which would only affect playback speed.
pub trait SetSpeed {
    /// Change the playback speed.
    ///
    /// 1.0 is the normal playback speed. 2.0 would be twice as fast, 0.5 would
    /// be half has fast.
    fn set_speed(&mut self, multiplier: f32);
}

/// A wrapper that adjusts the speed of the inner sound.
pub struct AdjustableSpeed<S: Sound> {
    inner: S,
    speed_adjustment: f32,
    speed_changed: bool,
}

impl<S> AdjustableSpeed<S>
where
    S: Sound,
{
    /// Wrap `inner` such that its speed can be adjusted.
    pub fn new(inner: S) -> Self {
        Self::new_with_speed(inner, 1.0)
    }

    /// Wrap `inner` such that its speed can be adjusted and set an initial
    /// adjustment.
    pub fn new_with_speed(inner: S, speed_adjustment: f32) -> Self {
        AdjustableSpeed {
            inner,
            speed_adjustment,
            speed_changed: false,
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

impl<S> Sound for AdjustableSpeed<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        let new_rate = (self.inner.sample_rate() as f32 * self.speed_adjustment).round() as u32;
        // Do not let the new rate be 0 which would cause issues
        u32::max(1, new_rate)
    }

    fn next_sample(&mut self) -> Result<crate::NextSample, crate::Error> {
        if self.speed_changed {
            self.speed_changed = false;
            return Ok(crate::NextSample::MetadataChanged);
        }
        self.inner.next_sample()
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S> SetSpeed for AdjustableSpeed<S>
where
    S: Sound,
{
    fn set_speed(&mut self, new: f32) {
        self.speed_changed = true;
        self.speed_adjustment = new;
    }
}

impl<S> AdjustableSpeed<S>
where
    S: Sound,
{
    /// Return the current speed multiplier. 1.0 is the default speed.
    pub fn speed(&self) -> f32 {
        self.speed_adjustment
    }
}

impl<S> SetPaused for AdjustableSpeed<S>
where
    S: Sound + SetPaused,
{
    fn set_paused(&mut self, paused: bool) {
        self.inner.set_paused(paused)
    }
}

impl<S> SetStopped for AdjustableSpeed<S>
where
    S: Sound + SetStopped,
{
    fn set_stopped(&mut self) {
        self.inner.set_stopped()
    }
}

impl<S> SetVolume for AdjustableSpeed<S>
where
    S: Sound + SetVolume,
{
    fn set_volume(&mut self, multiplier: f32) {
        self.inner.set_volume(multiplier)
    }
}

#[cfg(test)]
#[path = "./tests/adjustable_speed.rs"]
mod tests;
