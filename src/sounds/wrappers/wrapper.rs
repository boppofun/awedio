use crate::Sound;

use super::{AddSound, ClearSounds, SetPaused, SetSpeed, SetStopped, SetVolume};

/// Super trait that implements all traits that a wrapper Sound should
/// transparently pass through if implemented by the inner sound. If you have
/// a wrapper that should handle any of these traits specially besides just
/// passing through, then you should not implement this trait but implement all
/// of the traits individually.
pub trait Wrapper {
    /// The wrapped Sound type
    type Inner: Sound;

    /// Get a reference to the wrapped inner Sound.
    fn inner(&self) -> &Self::Inner;

    /// Get a mutable reference to the wrapped inner Sound.
    fn inner_mut(&mut self) -> &mut Self::Inner;

    /// Unwrap and return the previously wrapped Sound.
    fn into_inner(self) -> Self::Inner;
}

impl<S> SetPaused for S
where
    S: Wrapper,
    <S as Wrapper>::Inner: SetPaused,
{
    fn set_paused(&mut self, paused: bool) {
        self.inner_mut().set_paused(paused)
    }
}

impl<S> SetStopped for S
where
    S: Wrapper,
    <S as Wrapper>::Inner: SetStopped,
{
    fn set_stopped(&mut self) {
        self.inner_mut().set_stopped()
    }
}

impl<S> SetSpeed for S
where
    S: Wrapper,
    <S as Wrapper>::Inner: SetSpeed,
{
    fn set_speed(&mut self, new: f32) {
        self.inner_mut().set_speed(new)
    }
}

impl<S> SetVolume for S
where
    S: Wrapper,
    <S as Wrapper>::Inner: SetVolume,
{
    fn set_volume(&mut self, new: f32) {
        self.inner_mut().set_volume(new)
    }
}

impl<S> AddSound for S
where
    S: Wrapper,
    <S as Wrapper>::Inner: AddSound,
{
    fn add(&mut self, sound: Box<dyn crate::Sound>) {
        self.inner_mut().add(sound)
    }
}

impl<S> ClearSounds for S
where
    S: Wrapper,
    <S as Wrapper>::Inner: ClearSounds,
{
    fn clear(&mut self) {
        self.inner_mut().clear()
    }
}
