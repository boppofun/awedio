use crate::Sound;

use super::{SetPaused, SetSpeed, SetStopped};

/// A sound multiplied by a linear gain adjustment.
pub trait SetVolume {
    /// Change the gain multiplier.
    ///
    /// The samples are multiplied by `multiplier` so 1.0 would leave the Sound
    /// unchanged. 0.5 would reduce the sample values by half and 2.0 would
    /// double them (saturating if larger than the max value).
    ///
    /// These changes linear and 0.5 will not sound half as loud since
    fn set_volume(&mut self, multiplier: f32);
}

/// A wrapper that adjusts the gain of the inner sound.
pub struct AdjustableVolume<S: Sound> {
    inner: S,
    volume_adjustment: f32,
}

impl<S> AdjustableVolume<S>
where
    S: Sound,
{
    /// Wrap `inner` such that its gain can be adjusted.
    ///
    /// The value is set to 1.0 so no adjustment is made.
    ///
    /// See `set_volume`.
    pub fn new(inner: S) -> Self {
        AdjustableVolume {
            inner,
            volume_adjustment: 1.0,
        }
    }

    /// Wrap `inner` such that its volume can be adjusted and set an initial
    /// adjustment.
    ///
    /// See `set_volume`.
    pub fn new_with_volume(inner: S, volume_adjustment: f32) -> Self {
        AdjustableVolume {
            inner,
            volume_adjustment,
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

impl<S> Sound for AdjustableVolume<S>
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
        let next = self.inner.next_sample()?;
        Ok(match next {
            crate::NextSample::Sample(s) => {
                // Since Rust 1.45, the `as` keyword performs a *saturating cast*
                // when casting from float to int.
                let adjusted = (s as f32 * self.volume_adjustment) as i16;
                crate::NextSample::Sample(adjusted)
            }
            crate::NextSample::MetadataChanged
            | crate::NextSample::Paused
            | crate::NextSample::Finished => next,
        })
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S> AdjustableVolume<S>
where
    S: Sound,
{
    /// Return the current gain multiplier. 1.0 is the default multiplier.
    pub fn volume(&self) -> f32 {
        self.volume_adjustment
    }
}

impl<S> SetVolume for AdjustableVolume<S>
where
    S: Sound,
{
    fn set_volume(&mut self, new: f32) {
        self.volume_adjustment = new;
    }
}

impl<S> SetPaused for AdjustableVolume<S>
where
    S: Sound + SetPaused,
{
    fn set_paused(&mut self, paused: bool) {
        self.inner.set_paused(paused)
    }
}

impl<S> SetStopped for AdjustableVolume<S>
where
    S: Sound + SetStopped,
{
    fn set_stopped(&mut self) {
        self.inner.set_stopped()
    }
}

impl<S> SetSpeed for AdjustableVolume<S>
where
    S: Sound + SetSpeed,
{
    fn set_speed(&mut self, multiplier: f32) {
        self.inner.set_speed(multiplier)
    }
}

#[cfg(test)]
#[path = "./tests/adjustable_volume.rs"]
mod tests;
