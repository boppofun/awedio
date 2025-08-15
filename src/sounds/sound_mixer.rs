use super::wrappers::{AddSound, ChannelCountConverter, ClearSounds, SampleRateConverter};
use crate::sound::NextSample;
use crate::Sound;

type MixedSound = SampleRateConverter<ChannelCountConverter<Box<dyn Sound>>>;

/// Mix multiple sounds together to be played simultaneously.
///
/// The [Manager][crate::manager::Manager] contains a SoundMixer so you might
/// not need to crate one yourself but instead add multiple sounds on the
/// Manager.
///
/// If a Sound returns an Error from next_sample, the error is logged and the
/// Sound is dropped but other sounds keep playing.
pub struct SoundMixer {
    sounds: Vec<MixedSound>,
    paused_sounds: Vec<MixedSound>,
    output_channel_count: u16,
    output_sample_rate: u32,
    metadata_changed: bool,
    next_output_channel_idx: u16,
}

impl SoundMixer {
    /// Create a new empty sound mixer with an output channel count and sample
    /// rate that all added sounds will be converted to.
    pub fn new(output_channel_count: u16, output_sample_rate: u32) -> Self {
        SoundMixer {
            sounds: Vec::new(),
            paused_sounds: Vec::new(),
            output_channel_count,
            output_sample_rate,
            metadata_changed: false,
            next_output_channel_idx: 0,
        }
    }

    /// Set the output channel count and sample rate.
    /// Added sounds will be converted to the output values. Must only be called
    /// when the next sample is for the first channel in the frame.
    pub fn set_output_channel_count_and_sample_rate(
        &mut self,
        output_channel_count: u16,
        output_sample_rate: u32,
    ) {
        self.metadata_changed = true;

        self.output_channel_count = output_channel_count;
        self.output_sample_rate = output_sample_rate;

        // Now re-wrap all the sounds with the new values.

        // Move all sounds to a single vec for simplicity
        self.sounds.append(&mut self.paused_sounds);

        let mut old = Vec::new();
        std::mem::swap(&mut self.sounds, &mut old);
        for mixed_sound in old {
            let inner = mixed_sound.into_inner().into_inner();
            // add will rewrap the sound
            self.add(inner);
        }
    }
}

impl Sound for SoundMixer {
    fn channel_count(&self) -> u16 {
        self.output_channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.output_sample_rate
    }

    fn on_start_of_batch(&mut self) {
        // Attempt to grab from paused sounds again
        self.sounds.append(&mut self.paused_sounds);

        for sound in &mut self.sounds {
            sound.on_start_of_batch();
        }
    }

    /// Guaranteed to not return an Error.
    fn next_sample(&mut self) -> Result<crate::sound::NextSample, crate::Error> {
        if self.metadata_changed {
            assert!(self.next_output_channel_idx == 0);
            self.metadata_changed = false;
            return Ok(NextSample::MetadataChanged);
        }

        let mut output: i16 = 0;

        let mut to_remove = Vec::new();

        for (idx, sound) in self.sounds.iter_mut().enumerate() {
            loop {
                match sound.next_sample() {
                    Ok(NextSample::Sample(s)) => {
                        output = output.saturating_add(s);
                        break;
                    }
                    Ok(NextSample::MetadataChanged) => {
                        // We know that the channel_count and sample_rate haven't changed because
                        // we have wrapped the sound in converters. It is possible that the
                        // MetadataChanged implies we need to start over at the first channel.
                        // Normally however Metadata only change on the first sample of a frame
                        // so handle that by looping around and calling next_sample again
                        // immediately
                        if self.next_output_channel_idx != 0 {
                            // In the rare case we see MetadataChange not on
                            // the first channel, lets pause the sound until the
                            // next batch to avoid de-syncing the channels.
                            to_remove.push((idx, true));
                            break;
                        }
                    }
                    Ok(NextSample::Paused) => {
                        to_remove.push((idx, true));
                        break;
                    }
                    Ok(NextSample::Finished) => {
                        to_remove.push((idx, false));
                        break;
                    }
                    Err(e) => {
                        // TODO probably want to let applications subscribe to be notified of these
                        // errors
                        log::error!("dropping sound in SoundMixer which returned error: {}", e);
                        to_remove.push((idx, false));
                        break;
                    }
                }
            }
        }

        for (idx, paused) in to_remove.into_iter().rev() {
            let sound = self.sounds.swap_remove(idx);
            if paused {
                self.paused_sounds.push(sound);
            }
            // otherwise drop finished sound
        }

        self.next_output_channel_idx += 1;
        if self.next_output_channel_idx == self.output_channel_count {
            self.next_output_channel_idx = 0;
        }

        match (self.sounds.is_empty(), self.paused_sounds.is_empty()) {
            // We assume that we are finished since this sound has been handed
            // off to the Manager so new sounds can't be added without a
            // Controllable. If this is wrapped in a Controllable, the Finished
            // is changed to a Paused by the wrapper.
            (true, true) => {
                self.next_output_channel_idx = 0;
                Ok(NextSample::Finished)
            }
            (true, false) => {
                self.next_output_channel_idx = 0;
                Ok(NextSample::Paused)
            }
            (false, _) => Ok(NextSample::Sample(output)),
        }
    }
}

impl AddSound for SoundMixer {
    fn add(&mut self, sound: Box<dyn Sound>) {
        self.sounds.push(SampleRateConverter::new(
            ChannelCountConverter::new(sound, self.output_channel_count),
            self.output_sample_rate,
        ));
    }
}

impl ClearSounds for SoundMixer {
    /// Remove all audio sounds.
    fn clear(&mut self) {
        self.sounds.clear();
        self.paused_sounds.clear();
    }
}

#[cfg(test)]
#[path = "./tests/sound_mixer.rs"]
mod tests;
