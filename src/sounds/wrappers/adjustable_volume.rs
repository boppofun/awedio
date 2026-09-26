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
///
/// Volumes below 16.0 are applied with fixed point math.
pub struct AdjustableVolume<S: Sound> {
    inner: S,
    volume_adjustment: f32,
    gain: Gain,
}

/// Number of fractional bits for fixed point gains.
const FIXED_POINT_SHIFT: u32 = 12;
/// The largest fixed point gain such that `i16::MIN * gain` fits in an i32.
const MAX_FIXED_POINT_GAIN: i32 = u16::MAX as i32;

/// How the volume adjustment is applied to samples. Computed when the volume
/// changes so the per sample work is minimal.
#[derive(Debug, Clone, Copy)]
enum Gain {
    /// Samples are unchanged.
    Unity,
    /// All samples are 0.
    Mute,
    /// Multiply by the value and divide by 2^FIXED_POINT_SHIFT.
    FixedPoint(i32),
    /// Large or negative volumes.
    Float(f32),
}

impl Gain {
    fn new(volume_adjustment: f32) -> Gain {
        if volume_adjustment == 1.0 {
            return Gain::Unity;
        }
        if volume_adjustment == 0.0 {
            return Gain::Mute;
        }
        let fixed = (volume_adjustment * (1 << FIXED_POINT_SHIFT) as f32).round();
        if fixed >= 1.0 && fixed <= MAX_FIXED_POINT_GAIN as f32 {
            Gain::FixedPoint(fixed as i32)
        } else {
            Gain::Float(volume_adjustment)
        }
    }

    #[inline]
    fn apply(self, samples: &mut [i16]) {
        match self {
            Gain::Unity => (),
            Gain::Mute => samples.fill(0),
            Gain::FixedPoint(gain) => {
                for s in samples {
                    // Division truncates toward zero like the float cast
                    // below. The compiler turns this into shifts.
                    let adjusted = *s as i32 * gain / (1 << FIXED_POINT_SHIFT);
                    *s = adjusted.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                }
            }
            Gain::Float(gain) => {
                for s in samples {
                    // Since Rust 1.45, the `as` keyword performs a *saturating cast*
                    // when casting from float to int.
                    *s = (*s as f32 * gain) as i16;
                }
            }
        }
    }
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
        Self::new_with_volume(inner, 1.0)
    }

    /// Wrap `inner` such that its volume can be adjusted and set an initial
    /// adjustment.
    ///
    /// See `set_volume`.
    pub fn new_with_volume(inner: S, volume_adjustment: f32) -> Self {
        AdjustableVolume {
            inner,
            volume_adjustment,
            gain: Gain::new(volume_adjustment),
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

    #[inline]
    fn next_samples(&mut self, buf: &mut [i16]) -> crate::Filled {
        let filled = self.inner.next_samples(buf);
        self.gain.apply(&mut buf[..filled.written]);
        filled
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
        self.gain = Gain::new(new);
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
