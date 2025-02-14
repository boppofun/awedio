//! Wrappers add functionality or modify an existing Sound.
//!
//! This is the [Decorator pattern](https://en.wikipedia.org/wiki/Decorator_pattern).
//!
//! This are normally accessed from functions on the [Sound][crate::Sound] trait
//! instead of directly.

mod adjustable_speed;
mod adjustable_volume;
#[cfg(feature = "async")]
pub mod async_completion_notifier;
mod channel_count_converter;
mod completion_notifier;
mod controllable;
mod finish_after;
mod pausable;
mod sample_rate_converter;
mod stoppable;
mod wrapper;

pub use adjustable_speed::AdjustableSpeed;
pub use adjustable_speed::SetSpeed;
pub use adjustable_volume::AdjustableVolume;
pub use adjustable_volume::SetVolume;
#[cfg(feature = "async")]
pub use async_completion_notifier::AsyncCompletionNotifier;
pub use channel_count_converter::ChannelCountConverter;
pub use completion_notifier::CompletionNotifier;
pub use controllable::{Controllable, Controller};
pub use finish_after::FinishAfter;
pub use pausable::Pausable;
pub use pausable::SetPaused;
pub use sample_rate_converter::SampleRateConverter;
pub use stoppable::SetStopped;
pub use stoppable::Stoppable;
pub use wrapper::Wrapper;

/// A Sound which contains other sounds that can be added to it.
pub trait AddSound {
    /// Add a sound to be played. When or how the sound is played is
    /// implementation specific.
    fn add(&mut self, sound: Box<dyn crate::Sound>);
}

/// A Sound which contains other sounds and those Sounds can be cleared.
pub trait ClearSounds {
    /// Clear all sounds currently playing or scheduled to play.
    fn clear(&mut self);
}
