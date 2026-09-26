use super::wrappers::{AddSound, ChannelCountConverter, ClearSounds, SampleRateConverter};
use crate::{Filled, Sound, Stop};

type MixedSound = SampleRateConverter<ChannelCountConverter<Box<dyn Sound>>>;

/// Mix multiple sounds together to be played simultaneously.
///
/// The [Manager][crate::manager::Manager] contains a SoundMixer so you might
/// not need to crate one yourself but instead add multiple sounds on the
/// Manager.
///
/// If a Sound returns an Error from next_samples, the error is logged and the
/// Sound is dropped but other sounds keep playing.
pub struct SoundMixer {
    sounds: Vec<MixedSound>,
    paused_sounds: Vec<MixedSound>,
    output_channel_count: u16,
    output_sample_rate: u32,
    metadata_changed: bool,
    /// Where each sound is rendered before being added to the output.
    scratch: Vec<i16>,
}

/// Number of frames rendered for each sound at a time. The output buffer is
/// processed in chunks of this size.
const SCRATCH_NUM_FRAMES: usize = 256;

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
            scratch: Vec::new(),
        }
    }

    /// Set the output channel count and sample rate.
    /// Added sounds will be converted to the output values.
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
    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        if self.metadata_changed {
            self.metadata_changed = false;
            return Filled::stopped(0, Stop::MetadataChanged);
        }
        if self.sounds.is_empty() {
            return Filled::stopped(0, self.no_active_sounds_stop());
        }

        let channel_count = self.output_channel_count as usize;
        debug_assert!(buf.len().is_multiple_of(channel_count));
        let chunk_len = SCRATCH_NUM_FRAMES * channel_count;
        if self.scratch.len() != chunk_len {
            self.scratch = vec![0; chunk_len];
        }

        let buf_len = buf.len();
        let mut output_offset = 0;
        while output_offset < buf_len {
            let out = &mut buf[output_offset..(output_offset + chunk_len).min(buf_len)];
            out.fill(0);
            // The number of samples at the start of `out` that have been
            // written by at least one sound.
            let mut max_written = 0;
            let mut idx = 0;
            while idx < self.sounds.len() {
                let sound = &mut self.sounds[idx];
                let mut written = 0;
                let mut stop = None;
                // Sounds are wrapped in converters so the channel count and
                // sample rate never change and on MetadataChanged we can
                // keep pulling from the sound.
                while written < out.len() {
                    let filled = sound.next_samples(&mut self.scratch[written..out.len()]);
                    written += filled.written;
                    match filled.stop {
                        None | Some(Stop::MetadataChanged) => (),
                        Some(other) => {
                            stop = Some(other);
                            break;
                        }
                    }
                }
                for (o, s) in out[..written].iter_mut().zip(&self.scratch[..written]) {
                    *o = o.saturating_add(*s);
                }
                max_written = max_written.max(written);
                match stop {
                    None => idx += 1,
                    Some(Stop::MetadataChanged) => unreachable!(),
                    Some(Stop::Paused) => {
                        let sound = self.sounds.swap_remove(idx);
                        self.paused_sounds.push(sound);
                    }
                    Some(Stop::Finished) => {
                        self.sounds.swap_remove(idx);
                    }
                    Some(Stop::Error(e)) => {
                        // TODO probably want to let applications subscribe to be notified of these
                        // errors
                        log::error!("dropping sound in SoundMixer which returned error: {}", e);
                        self.sounds.swap_remove(idx);
                    }
                }
            }
            if self.sounds.is_empty() {
                // Keep the samples of sounds that stopped part way through
                // this chunk.
                return Filled::stopped(output_offset + max_written, self.no_active_sounds_stop());
            }
            output_offset += out.len();
        }
        Filled::all(buf_len)
    }
}

impl SoundMixer {
    fn no_active_sounds_stop(&self) -> Stop {
        // We assume that we are finished if there are no sounds since this
        // sound has been handed off to the Manager so new sounds can't be added
        // without a Controllable. If this is wrapped in a Controllable, the
        // Finished is changed to a Paused by the wrapper.
        if self.paused_sounds.is_empty() {
            Stop::Finished
        } else {
            Stop::Paused
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
