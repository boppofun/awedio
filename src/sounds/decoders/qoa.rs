use crate::{Filled, Sound, Stop};
use qoaudio::{DecodeError, QoaDecoder as RawQoaDecoder};
use std::io::Read;

/// Decoder for the [QOA](https://qoaformat.org/) format.
///
/// If the data ends part way through a frame (e.g. a truncated file), the
/// samples decoded so far are returned followed by Finished. Samples decoded
/// by the call of next_samples that reached the end might be dropped.
pub struct QoaDecoder<R>
where
    R: Read + Send,
{
    raw_decoder: RawQoaDecoder<R>,
    sample_rate: u32,
    channel_count: u16,
}

impl<R> QoaDecoder<R>
where
    R: Read + Send,
{
    /// Attempts to decode the data as QOA audio.
    ///
    /// QoaDecoder makes many small reads so wrapping a `File` with a
    /// `BufReader` is recommended.
    pub fn new(data: R) -> Result<QoaDecoder<R>, DecodeError> {
        let mut raw_decoder = RawQoaDecoder::new(data)?;
        if raw_decoder.current_frame_header().num_channels == 0 {
            // In streaming mode the first frame header has not been read yet.
            raw_decoder.next_frame()?.ok_or(DecodeError::NoSamples)?;
        }
        let first_frame = raw_decoder.current_frame_header();
        let sample_rate = first_frame.sample_rate;
        let channel_count = first_frame.num_channels as u16;

        Ok(QoaDecoder {
            raw_decoder,
            sample_rate,
            channel_count,
        })
    }

    /// Return the wrapped Reader
    pub fn into_inner(self) -> R {
        self.raw_decoder.into_inner()
    }
}

impl<R> Sound for QoaDecoder<R>
where
    R: Read + Send,
{
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let mut written = 0;
        while written < buf.len() {
            match self.raw_decoder.decode_into(&mut buf[written..]) {
                Ok(0) => (),
                Ok(n) => {
                    written += n;
                    continue;
                }
                Err(e) => return Filled::stopped(written, stop_for_error(e)),
            }
            // The current frame is done.
            match self.raw_decoder.next_frame() {
                Ok(None) => return Filled::stopped(written, Stop::Finished),
                Ok(Some(f)) => {
                    if f.num_channels as u16 != self.channel_count
                        || f.sample_rate != self.sample_rate
                    {
                        self.channel_count = f.num_channels.into();
                        self.sample_rate = f.sample_rate;
                        return Filled::stopped(written, Stop::MetadataChanged);
                    }
                }
                Err(e) => return Filled::stopped(written, stop_for_error(e)),
            }
        }
        Filled::all(written)
    }

    fn on_start_of_batch(&mut self) {}
}

/// Treat data ending part way through a frame as the end of the sound.
fn stop_for_error(e: DecodeError) -> Stop {
    match e {
        DecodeError::IoError(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Stop::Finished,
        e => Stop::Error(e.into()),
    }
}

impl From<DecodeError> for crate::Error {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::IoError(e) => e.into(),
            DecodeError::NotQoaFile
            | DecodeError::NoSamples
            | DecodeError::InvalidFrameHeader
            | DecodeError::IncompatibleFrame => crate::Error::FormatError(Box::new(value)),
        }
    }
}

#[cfg(test)]
#[path = "./tests/qoa.rs"]
mod tests;
