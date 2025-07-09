use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use crate::{
    sounds::{
        wrappers::{
            AdjustableSpeed, AdjustableVolume, Controllable, Controller, FinishAfter, Pausable,
            SetPaused, Stoppable,
        },
        MemorySound,
    },
    utils,
};

/// A provider of audio samples.
///
/// This is the foundational trait of this crate. A `Box<dyn Sound>` can be
/// played on a [Manager][crate::manager::Manager]. Sounds can be wrapped to
/// modify the inner sound, often by using helper functions of this trait
/// (e.g. [pausable][Sound::pausable]).
pub trait Sound: Send {
    /// Returns the number of channels.
    fn channel_count(&self) -> u16;

    /// Returns the number of samples per second for each channel for this sound
    /// (e.g. 48,000).
    fn sample_rate(&self) -> u32;

    /// Retrieve the next sample or notification if something has changed.
    /// The first sample is for the first channel and the second is the for
    /// second and so on until channel_count and then wraps back to the first
    /// channel. If any NextSample variant besides `Sample` is returned then
    /// the following `NextSample::Sample` is for the first channel. If a Sound
    /// has returned `Paused` it is expected that the consumer will call
    /// next_sample again in the future. If a Sound has returned `Finished` it
    /// is not expected for the consumer to call next_sample again but if called
    /// `Finished` will normally be returned again. After Finished has been
    /// returned, channel_count() and sample_rate() may return different values
    /// without MetadataChanged being returned.
    ///
    /// If an error is returned it is not specified what will happen if
    /// next_sample is called again. Individual implementations can specify
    /// which errors are recoverable if any. Most consumers will either pass the
    /// error up or log the error and stop playing the sound (e.g. `SoundMixer`
    /// and `SoundList`).
    fn next_sample(&mut self) -> Result<NextSample, crate::Error>;

    /// Called whenever a new batch of audio samples is requested by the
    /// backend.
    ///
    /// This is a good place to put code that needs to run fairly frequently,
    /// but not for every single audio sample.
    fn on_start_of_batch(&mut self);

    /// Returns the next sample for all channels.
    ///
    /// It is the callers responsibility to ensure this function is only called
    /// at the start of a frame (i.e. the first channel is the next to be
    /// returned from next_sample).
    ///
    /// If an Error, `Paused`, `Finished`, or `MetadataChanged` are encountered
    /// while collecting samples, an Err(Ok(NextSample)) of that variant
    /// will be returned and any previously collected samples are lost.
    /// Err(Ok(NextSample::Sample)) will never be returned. If an error is
    /// encountered Err(Err(error::Error)) is returned.
    fn next_frame(&mut self) -> Result<Vec<i16>, Result<NextSample, crate::Error>> {
        let mut samples = Vec::with_capacity(self.channel_count() as usize);
        self.append_next_frame_to(&mut samples)?;
        Ok(samples)
    }

    /// Same as `next_frame` but samples are appended into an existing Vec.
    ///
    /// Any existing data is left unmodified.
    fn append_next_frame_to(
        &mut self,
        samples: &mut Vec<i16>,
    ) -> Result<(), Result<NextSample, crate::Error>> {
        for _ in 0..self.channel_count() {
            let next = self.next_sample();
            match next {
                Ok(NextSample::Sample(s)) => samples.push(s),
                Ok(NextSample::MetadataChanged)
                | Ok(NextSample::Paused)
                | Ok(NextSample::Finished)
                | Err(_) => return Err(next),
            }
        }
        Ok(())
    }

    /// Read the entire sound into memory. MemorySound can be cloned for
    /// efficient reuse. See [MemorySound::from_sound].
    fn into_memory_sound(self) -> Result<MemorySound, crate::Error>
    where
        Self: Sized,
    {
        MemorySound::from_sound(self)
    }

    /// Read the entire sound into memory and loop indefinitely.
    ///
    /// If you do not want to read the entire sound into memory see
    /// [SoundsFromFn][crate::sounds::SoundsFromFn] as an alternative.
    fn loop_from_memory(self) -> Result<MemorySound, crate::Error>
    where
        Self: Sized,
    {
        let mut to_return = MemorySound::from_sound(self)?;
        to_return.set_looping(true);
        Ok(to_return)
    }

    /// Allow this sound to be controlled after it has started playing with a
    /// [`Controller`].
    ///
    /// What can be controlled depends on the Sound type (e.g. set_volume).
    fn controllable(self) -> (Controllable<Self>, Controller<Self>)
    where
        Self: Sized,
    {
        Controllable::new(self)
    }

    /// Get notified via a [tokio::sync::oneshot::Receiver] when this sound
    /// has Finished.
    #[cfg(feature = "async")]
    fn with_async_completion_notifier(
        self,
    ) -> (
        crate::sounds::wrappers::AsyncCompletionNotifier<Self>,
        tokio::sync::oneshot::Receiver<()>,
    )
    where
        Self: Sized,
    {
        crate::sounds::wrappers::AsyncCompletionNotifier::new(self)
    }

    /// Get notified via a [std::sync::mpsc::Receiver] when this sound
    /// has Finished.
    fn with_completion_notifier(
        self,
    ) -> (
        crate::sounds::wrappers::CompletionNotifier<Self>,
        std::sync::mpsc::Receiver<()>,
    )
    where
        Self: Sized,
    {
        crate::sounds::wrappers::CompletionNotifier::new(self)
    }

    /// Allow the volume of the sound to be adjustable with `set_volume`.
    fn with_adjustable_volume(self) -> AdjustableVolume<Self>
    where
        Self: Sized,
    {
        AdjustableVolume::new(self)
    }

    /// Allow the volume of the sound to be adjustable with `set_volume` and set
    /// the initial volume adjustment.
    fn with_adjustable_volume_of(self, volume_adjustment: f32) -> AdjustableVolume<Self>
    where
        Self: Sized,
    {
        AdjustableVolume::new_with_volume(self, volume_adjustment)
    }

    /// Allow the speed of the sound to be adjustable with `set_speed`.
    ///
    /// This adjusts both speed and pitch.
    fn with_adjustable_speed(self) -> AdjustableSpeed<Self>
    where
        Self: Sized,
    {
        AdjustableSpeed::new(self)
    }

    /// Allow the speed of the sound to be adjustable with `set_speed` and set
    /// the initial speed adjustment.
    ///
    /// This adjusts both speed and pitch.
    fn with_adjustable_speed_of(self, speed_adjustment: f32) -> AdjustableSpeed<Self>
    where
        Self: Sized,
    {
        AdjustableSpeed::new_with_speed(self, speed_adjustment)
    }

    /// Allow for the sound to be pausable with `set_paused`. Starts unpaused.
    fn pausable(self) -> Pausable<Self>
    where
        Self: Sized,
    {
        Pausable::new(self)
    }

    /// Allow for the sound to be pausable with `set_paused`. Starts paused.
    fn paused(self) -> Pausable<Self>
    where
        Self: Sized,
    {
        let mut to_return = Pausable::new(self);
        to_return.set_paused(true);
        to_return
    }

    /// Allow for the sound to be stoppable with `set_stopped`.
    /// A stopped sound returns `Finished`.
    fn stoppable(self) -> Stoppable<Self>
    where
        Self: Sized,
    {
        Stoppable::new(self)
    }

    /// Play the first `duration` of the sound, then finish even if samples
    /// remain.
    ///
    /// See [FinishAfter].
    fn finish_after(self, duration: Duration) -> FinishAfter<Self>
    where
        Self: Sized,
    {
        FinishAfter::new(self, duration)
    }

    /// Skip the next `duration` of samples.
    ///
    /// This is done by calling next_sample repeatedly.
    ///
    /// Returns true if all samples were successfully skipped, false if a Paused
    /// or Finished were encountered first. MetadataChanged events are handled
    /// correctly but are not returned.
    fn skip(&mut self, duration: Duration) -> Result<bool, crate::Error> {
        let mut current_channel_count = self.channel_count();
        let mut current_sample_rate = self.sample_rate();
        let mut num_samples_remaining =
            utils::duration_to_num_samples(duration, current_channel_count, current_sample_rate);

        while num_samples_remaining > 0 {
            let next = self.next_sample()?;
            match next {
                NextSample::Sample(_) => {
                    num_samples_remaining -= 1;
                }
                NextSample::MetadataChanged => {
                    let new_channel_count = self.channel_count();
                    let new_sample_rate = self.sample_rate();
                    if new_channel_count != current_channel_count
                        || new_sample_rate != current_sample_rate
                    {
                        num_samples_remaining = utils::convert_num_samples(
                            num_samples_remaining,
                            current_channel_count,
                            current_sample_rate,
                            new_channel_count,
                            new_sample_rate,
                        );
                        current_channel_count = new_channel_count;
                        current_sample_rate = new_sample_rate;
                    }
                }
                NextSample::Paused => return Ok(false),
                NextSample::Finished => return Ok(false),
            }
        }
        Ok(true)
    }
}

/// The result of [Sound::next_sample]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NextSample {
    /// A sample for one channel. Channels are interleaved. The first sample is
    /// for the first channel and so forth and repeats (e.g. L-R-L-R-L-R).
    Sample(i16),

    /// The number of channels or the sample rate has changed. Continue to
    /// retrieve samples afterward. The next sample will always be for the
    /// first track regardless of what track was next
    // before this value was returned.
    MetadataChanged,

    /// No more samples for now. More might come later. It is expected that the
    /// Sound will not be pulled again during this batch of samples.
    Paused,

    /// All samples have been retrieved and no more will come.
    Finished,
}

impl Sound for Box<dyn Sound> {
    fn on_start_of_batch(&mut self) {
        self.deref_mut().on_start_of_batch()
    }

    fn channel_count(&self) -> u16 {
        self.deref().channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.deref().sample_rate()
    }

    fn next_sample(&mut self) -> Result<NextSample, crate::Error> {
        self.deref_mut().next_sample()
    }
}

#[cfg(test)]
#[path = "./tests/sound.rs"]
mod tests;
