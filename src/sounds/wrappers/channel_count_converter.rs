use crate::{Filled, Sound, Stop};

use super::Wrapper;

/// Convert a Sound to have a specified number of output channels.
/// For example convert a mono sound to stereo or vice versa.
pub struct ChannelCountConverter<S: Sound> {
    inner: S,
    to_count: u16,
    converter_type: ConverterType,
    /// Samples from inner when inner has more channels than the output.
    input_buffer: Vec<i16>,
}

#[derive(Clone, Copy)]
enum ConverterType {
    PassThrough,
    MonoToStereo,
    StereoToMono,
}

impl<S> ChannelCountConverter<S>
where
    S: Sound,
{
    /// Wrap `inner` such that it will output `to_count` channels.
    pub fn new(inner: S, to_count: u16) -> ChannelCountConverter<S> {
        let converter_type = Self::get_type(inner.channel_count(), to_count);

        ChannelCountConverter {
            inner,
            to_count,
            converter_type,
            input_buffer: Vec::new(),
        }
    }

    fn get_type(from_count: u16, to_count: u16) -> ConverterType {
        if from_count == to_count {
            ConverterType::PassThrough
        } else if from_count == 1 && to_count == 2 {
            ConverterType::MonoToStereo
        } else if from_count == 2 && to_count == 1 {
            ConverterType::StereoToMono
        } else {
            // Can implement more conversions like
            // https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Basic_concepts_behind_Web_Audio_API#up-mixing_and_down-mixing
            todo!(
                "ChannelCountConverter for {} to {} channels not implemented.",
                from_count,
                to_count
            );
        }
    }

    // We could save the metadata of the inner Source and only return MetadataChange
    // if the metadata change is something we can't handle (i.e. a Rate Change).
    fn handle_possible_channel_count_change(&mut self, filled: &Filled) {
        if let Some(Stop::MetadataChanged) = filled.stop {
            let from_count = self.inner.channel_count();
            self.converter_type = Self::get_type(from_count, self.to_count);
        }
    }

    /// Unwrap the inner Sound.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S> Sound for ChannelCountConverter<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.to_count
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let filled = match self.converter_type {
            ConverterType::PassThrough => self.inner.next_samples(buf),
            ConverterType::MonoToStereo => {
                let num_frames = buf.len() / 2;
                let filled = self.inner.next_samples(&mut buf[..num_frames]);
                let written = filled.written;
                // Expand in place starting from the end so we do not
                // overwrite samples we have not yet copied.
                for i in (0..written).rev() {
                    let sample = buf[i];
                    buf[i * 2] = sample;
                    buf[i * 2 + 1] = sample;
                }
                Filled {
                    written: written * 2,
                    stop: filled.stop,
                }
            }
            ConverterType::StereoToMono => {
                let input_len = buf.len() * 2;
                if self.input_buffer.len() < input_len {
                    self.input_buffer.resize(input_len, 0);
                }
                let filled = self.inner.next_samples(&mut self.input_buffer[..input_len]);
                let written = filled.written / 2;
                for (out, frame) in buf[..written]
                    .iter_mut()
                    .zip(self.input_buffer[..filled.written].as_chunks::<2>().0)
                {
                    // Get the average of the two
                    *out = ((frame[0] as i32 + frame[1] as i32) / 2) as i16;
                }
                Filled {
                    written,
                    stop: filled.stop,
                }
            }
        };
        self.handle_possible_channel_count_change(&filled);
        filled
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S: Sound> Wrapper for ChannelCountConverter<S> {
    type Inner = S;

    fn inner(&self) -> &S {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }

    fn into_inner(self) -> S {
        self.inner
    }
}

#[cfg(test)]
#[path = "./tests/channel_count_converter.rs"]
mod tests;
