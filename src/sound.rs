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
///
/// Samples are produced in batches by [next_samples][Sound::next_samples].
/// For tests and other rare callers that want a single sample or frame at a
/// time see [SampleBySample][crate::sounds::wrappers::SampleBySample].
pub trait Sound: Send {
    /// Returns the number of channels.
    fn channel_count(&self) -> u16;

    /// Returns the number of samples per second for each channel for this sound
    /// (e.g. 48,000).
    fn sample_rate(&self) -> u32;

    /// Fill `buf` with the next samples.
    ///
    /// Samples are interleaved by channel: the first sample is for the first
    /// channel, the second is for the second channel and so on until
    /// channel_count and then wraps back to the first channel.
    ///
    /// The caller must pass a `buf` whose length is a multiple of
    /// `channel_count()`. The Sound writes to `buf[..written]` where `written`
    /// is always a multiple of `channel_count()` (i.e. only whole frames are
    /// written). The caller must not assume anything about the contents of
    /// `buf` after `written`.
    ///
    /// If `stop` is None, `written` is equal to `buf.len()`. Otherwise `stop`
    /// describes why fewer samples may have been written. Samples written
    /// before a stop are valid and should be played.
    ///
    /// * [Stop::MetadataChanged]: the channel count and/or sample rate may
    ///   have changed after the written samples. The caller should query
    ///   `channel_count()` and `sample_rate()` again and may call
    ///   `next_samples` again immediately. A Sound must not keep returning
    ///   MetadataChanged without writing samples (or returning another stop)
    ///   since callers such as SoundMixer loop until they get samples.
    /// * [Stop::Paused]: no more samples for now. More might come later. It
    ///   is expected that the Sound will not be pulled again until the next
    ///   batch.
    /// * [Stop::Finished]: all samples have been retrieved and no more will
    ///   come. If called again `Finished` will normally be returned again.
    ///   After Finished has been returned, channel_count() and sample_rate()
    ///   may return different values without MetadataChanged being returned.
    /// * [Stop::Error]: it is not specified what will happen if next_samples
    ///   is called again. Individual implementations can specify which errors
    ///   are recoverable if any. Most consumers will either pass the error up
    ///   or log the error and stop playing the sound (e.g. `SoundMixer` and
    ///   `SoundList`).
    ///
    /// [on_start_of_batch][Sound::on_start_of_batch] is called once per batch
    /// and then `next_samples` may be called one or more times for that batch.
    fn next_samples(&mut self, buf: &mut [i16]) -> Filled;

    /// Called whenever a new batch of audio samples is requested by the
    /// backend.
    ///
    /// This is a good place to put code that needs to run fairly frequently,
    /// but not for every single audio sample.
    fn on_start_of_batch(&mut self);

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
    /// This is done by calling next_samples repeatedly and discarding the
    /// samples.
    ///
    /// Returns true if all samples were successfully skipped, false if a Paused
    /// or Finished were encountered first. MetadataChanged events are handled
    /// correctly but are not returned.
    fn skip(&mut self, duration: Duration) -> Result<bool, crate::Error> {
        const SCRATCH_LEN: usize = 256;
        let mut scratch = [0_i16; SCRATCH_LEN];
        // Only used for sounds with more channels than fit in scratch.
        let mut large_scratch = Vec::new();
        let mut current_channel_count = self.channel_count();
        let mut current_sample_rate = self.sample_rate();
        let mut num_frames_remaining =
            utils::duration_to_num_samples(duration, 1, current_sample_rate);

        while num_frames_remaining > 0 {
            let channel_count = current_channel_count as usize;
            let buf: &mut [i16] = if channel_count <= SCRATCH_LEN {
                &mut scratch
            } else {
                large_scratch.resize(channel_count, 0);
                &mut large_scratch
            };
            let max_frames = (buf.len() / channel_count) as u64;
            let frames = num_frames_remaining.min(max_frames) as usize;
            let filled = self.next_samples(&mut buf[..frames * channel_count]);
            num_frames_remaining -= (filled.written / channel_count) as u64;
            match filled.stop {
                None => (),
                Some(Stop::MetadataChanged) => {
                    let new_sample_rate = self.sample_rate();
                    if new_sample_rate != current_sample_rate {
                        num_frames_remaining = utils::convert_num_samples(
                            num_frames_remaining,
                            1,
                            current_sample_rate,
                            1,
                            new_sample_rate,
                        );
                    }
                    current_channel_count = self.channel_count();
                    current_sample_rate = new_sample_rate;
                }
                Some(Stop::Paused) | Some(Stop::Finished) => return Ok(false),
                Some(Stop::Error(e)) => return Err(e),
            }
        }
        Ok(true)
    }
}

/// The result of [Sound::next_samples].
#[derive(Debug)]
#[must_use]
pub struct Filled {
    /// The number of samples written to the start of the buffer. Always a
    /// multiple of the channel count.
    pub written: usize,
    /// Why the buffer was not completely filled or None if it was completely
    /// filled.
    pub stop: Option<Stop>,
}

impl Filled {
    /// The entire buffer of length `written` was filled.
    #[inline]
    pub fn all(written: usize) -> Filled {
        Filled {
            written,
            stop: None,
        }
    }

    /// `written` samples were written and then `stop` was encountered.
    #[inline]
    pub fn stopped(written: usize, stop: Stop) -> Filled {
        Filled {
            written,
            stop: Some(stop),
        }
    }
}

/// Why a [Sound] stopped writing samples in [Sound::next_samples].
#[derive(Debug)]
pub enum Stop {
    /// The number of channels or the sample rate might have changed. Continue
    /// to retrieve samples afterward with the new metadata.
    MetadataChanged,

    /// No more samples for now. More might come later. It is expected that the
    /// Sound will not be pulled again during this batch of samples.
    Paused,

    /// All samples have been retrieved and no more will come.
    Finished,

    /// An error occurred. See [Sound::next_samples] for details.
    Error(crate::Error),
}

impl From<crate::Error> for Stop {
    fn from(e: crate::Error) -> Self {
        Stop::Error(e)
    }
}

impl Sound for Box<dyn Sound> {
    #[inline]
    fn on_start_of_batch(&mut self) {
        self.deref_mut().on_start_of_batch()
    }

    #[inline]
    fn channel_count(&self) -> u16 {
        self.deref().channel_count()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.deref().sample_rate()
    }

    #[inline]
    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        self.deref_mut().next_samples(buf)
    }
}

#[cfg(test)]
#[path = "./tests/sound.rs"]
mod tests;
