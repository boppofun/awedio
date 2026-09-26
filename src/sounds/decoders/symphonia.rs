use crate::{Filled, Sound, Stop};
use symphonia::core::audio::conv::FromSample;
use symphonia::core::audio::sample::Sample;
use symphonia::core::audio::{Audio, AudioBuffer, Channels, GenericAudioBufferRef};
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions, CODEC_ID_NULL_AUDIO};
use symphonia::core::codecs::CodecParameters;
use symphonia::core::common::Limit;
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;

/// Decode formats using the Symphonia crate decoders.
pub struct SymphoniaDecoder {
    sample_rate: u32,

    decoder: Box<dyn AudioDecoder>,
    format: Box<dyn FormatReader>,

    channels: Channels,
    track_id: u32,
    /// The next frame of the last decoded buffer to output.
    next_frame_idx: usize,
}

impl SymphoniaDecoder {
    /// A decoder for the first track in data that has a recognized codec.
    ///
    /// The track may have multiple channels.
    pub fn new(
        data: Box<dyn MediaSource>,
        extension: Option<&str>,
    ) -> Result<SymphoniaDecoder, Error> {
        let mss = MediaSourceStream::new(data, Default::default());

        let mut hint = Hint::new();
        if let Some(extension) = extension {
            hint.with_extension(extension);
        }
        let meta_opts = MetadataOptions::default()
            .limit_tag_bytes(Limit::Maximum(1))
            .limit_visual_bytes(Limit::Maximum(1));
        let fmt_opts: FormatOptions = Default::default();
        let format = symphonia::default::get_probe().probe(&hint, mss, fmt_opts, meta_opts)?;

        // Find the first audio track with a known (decodable) codec.
        let track = format
            .tracks()
            .iter()
            .find(|t| {
                matches!(&t.codec_params, Some(CodecParameters::Audio(p)) if p.codec != CODEC_ID_NULL_AUDIO)
            })
            .ok_or(Error::Unsupported(
                "No track with a supported codec was found",
            ))?;
        let track_id = track.id;
        let audio_params = match &track.codec_params {
            Some(CodecParameters::Audio(p)) => p.clone(),
            _ => unreachable!(),
        };

        let dec_opts: AudioDecoderOptions = Default::default();
        let decoder =
            symphonia::default::get_codecs().make_audio_decoder(&audio_params, &dec_opts)?;

        let mut decoder = SymphoniaDecoder {
            sample_rate: 1000,
            decoder,
            format,
            channels: Channels::None,
            track_id,
            next_frame_idx: 0,
        };
        // Ignore metadata changed since no one has seen the old values
        let _ = decoder.decode_next_packet();
        Ok(decoder)
    }
}

impl Sound for SymphoniaDecoder {
    fn channel_count(&self) -> u16 {
        self.channels.count().try_into().unwrap()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let channel_count = self.channels.count();
        if channel_count == 0 {
            return Filled::stopped(0, Stop::Finished);
        }
        let mut written = 0;
        loop {
            let buf_ref = self.decoder.last_decoded();
            let frames_available = buf_ref.frames().saturating_sub(self.next_frame_idx);
            let num_frames = frames_available.min((buf.len() - written) / channel_count);
            copy_interleaved_from_ref(
                &buf_ref,
                self.next_frame_idx,
                &mut buf[written..written + num_frames * channel_count],
                channel_count,
            );
            self.next_frame_idx += num_frames;
            written += num_frames * channel_count;
            if written == buf.len() {
                return Filled::all(written);
            }
            match self.decode_next_packet() {
                Ok(Some(true)) => return Filled::stopped(written, Stop::MetadataChanged),
                Ok(Some(false)) => (),
                Ok(None) => return Filled::stopped(written, Stop::Finished),
                Err(e) => return Filled::stopped(written, Stop::Error(e.into())),
            };
        }
    }

    fn on_start_of_batch(&mut self) {}
}

impl SymphoniaDecoder {
    fn decode_next_packet(&mut self) -> Result<Option<bool>, Error> {
        loop {
            let Some(packet) = self.format.next_packet()? else {
                return Ok(None);
            };
            // We don't currently use the metadata but pop it off so it does not take
            // memory.
            while !self.format.metadata().is_latest() {
                self.format.metadata().pop();
            }
            if packet.track_id != self.track_id {
                continue;
            }

            // According to the Symphonia, some errors are indeed recoverable:
            let buf_ref = match self.decoder.decode(&packet) {
                Ok(buf_ref) => buf_ref,
                // Recoverable, but this packet is void. Expect weird noises!
                Err(Error::DecodeError(e)) => {
                    log::warn!("DecodeError while decoding stream: {}", e);
                    continue;
                }
                // Reset required, which is handled correctly by this decoder
                Err(Error::ResetRequired) => continue,
                // All other errors are unrecoverable
                Err(e) => return Err(e),
            };

            self.next_frame_idx = 0;
            let mut metadata_changed = false;
            if buf_ref.spec().channels() != &self.channels {
                self.channels = buf_ref.spec().channels().clone();
                metadata_changed = true;
            }
            if buf_ref.spec().rate() != self.sample_rate {
                self.sample_rate = buf_ref.spec().rate();
                metadata_changed = true;
            }
            return Ok(Some(metadata_changed));
        }
    }
}

fn copy_interleaved_from_ref(
    buffer: &GenericAudioBufferRef,
    first_frame: usize,
    out: &mut [i16],
    channel_count: usize,
) {
    match buffer {
        GenericAudioBufferRef::U8(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::U16(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::U24(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::U32(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::S8(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::S16(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::S24(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::S32(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::F32(b) => copy_interleaved(b, first_frame, out, channel_count),
        GenericAudioBufferRef::F64(b) => copy_interleaved(b, first_frame, out, channel_count),
    }
}

fn copy_interleaved<S: Sample>(
    buffer: &AudioBuffer<S>,
    first_frame: usize,
    out: &mut [i16],
    channel_count: usize,
) where
    i16: FromSample<S>,
{
    let num_frames = out.len() / channel_count;
    for channel_idx in 0..channel_count {
        let plane = &buffer.plane(channel_idx).unwrap()[first_frame..first_frame + num_frames];
        for (frame_idx, sample) in plane.iter().enumerate() {
            out[frame_idx * channel_count + channel_idx] = FromSample::from_sample(*sample);
        }
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::IoError(e) => e.into(),
            e => crate::Error::FormatError(Box::new(e)),
        }
    }
}

#[cfg(test)]
#[path = "./tests/symphonia.rs"]
mod tests;
