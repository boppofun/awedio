use std::io::Read;

use crate::{Filled, Sound, Stop};

use hound::{SampleFormat, WavReader};

// Originally based off Decoder from Rodio.

/// Decoder for the WAV format.
pub struct WavDecoder<R>
where
    R: Read + Send,
{
    reader: WavReader<R>,
    sample_rate: u32,
    channel_count: u16,
}

impl<R> WavDecoder<R>
where
    R: Read + Send,
{
    /// Attempts to decode the data as WAV.
    pub fn new(data: R) -> Result<WavDecoder<R>, hound::Error> {
        let reader = WavReader::new(data)?;
        let spec = reader.spec();

        let sample_rate = spec.sample_rate;
        let channel_count = spec.channels;

        Ok(WavDecoder {
            reader,
            sample_rate,
            channel_count,
        })
    }

    /// Return the wrapped Reader
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}

impl<R> Sound for WavDecoder<R>
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
        let spec = self.reader.spec();
        let ch = self.channel_count as usize;
        match (spec.sample_format, spec.bits_per_sample) {
            (SampleFormat::Float, 32) => fill(self.reader.samples(), buf, ch, f32_to_i16),
            (SampleFormat::Int, 8) => fill(self.reader.samples(), buf, ch, i8_to_i16),
            (SampleFormat::Int, 16) => fill(self.reader.samples(), buf, ch, |s: i16| s),
            (SampleFormat::Int, 24) => fill(self.reader.samples(), buf, ch, i24_to_i16),
            (SampleFormat::Int, 32) => fill(self.reader.samples(), buf, ch, i32_to_i16),
            (sample_format, bits_per_sample) => {
                unimplemented!("wav spec: {:?}, {}", sample_format, bits_per_sample)
            }
        }
    }

    fn on_start_of_batch(&mut self) {}
}

/// Fill `buf` from `samples`. If `samples` stops part way through a frame, the
/// samples of that partial frame are dropped.
fn fill<T>(
    mut samples: impl Iterator<Item = Result<T, hound::Error>>,
    buf: &mut [i16],
    channel_count: usize,
    convert: impl Fn(T) -> i16,
) -> Filled {
    for (idx, out) in buf.iter_mut().enumerate() {
        let stop = match samples.next() {
            Some(Ok(sample)) => {
                *out = convert(sample);
                continue;
            }
            Some(Err(e)) => Stop::Error(e.into()),
            None => Stop::Finished,
        };
        return Filled::stopped(idx - idx % channel_count, stop);
    }
    Filled::all(buf.len())
}

// Lossy
fn f32_to_i16(f: f32) -> i16 {
    (f.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

fn i8_to_i16(i: i8) -> i16 {
    i as i16 * 256
}

// Lossy
fn i24_to_i16(i: i32) -> i16 {
    (i >> 8) as i16
}

// Lossy
fn i32_to_i16(i: i32) -> i16 {
    (i >> 16) as i16
}

impl From<hound::Error> for crate::Error {
    fn from(value: hound::Error) -> Self {
        match value {
            hound::Error::IoError(e) => e.into(),
            hound::Error::FormatError(_)
            | hound::Error::TooWide
            | hound::Error::UnfinishedSample
            | hound::Error::Unsupported
            | hound::Error::InvalidSampleFormat => crate::Error::FormatError(Box::new(value)),
        }
    }
}

#[cfg(test)]
#[path = "./tests/wav.rs"]
mod tests;
