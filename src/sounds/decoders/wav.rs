use std::io::Read;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{Filled, Sound, Stop};

use hound::{SampleFormat, WavReader};

// Originally based off Decoder from Rodio.

/// Decoder for the WAV format.
///
/// If the data ends before all samples of the header have been read (e.g. a
/// truncated file), the samples read so far are returned followed by
/// Finished.
pub struct WavDecoder<R>
where
    R: Read + Send,
{
    reader: WavReader<EofTracker<R>>,
    /// Shared with the EofTracker since WavReader does not give access to
    /// its reader.
    reached_eof: Arc<AtomicBool>,
    sample_rate: u32,
    channel_count: u16,
}

impl<R> WavDecoder<R>
where
    R: Read + Send,
{
    /// Attempts to decode the data as WAV.
    ///
    /// Returns an error if the sample format is not supported or the sample
    /// rate is 0.
    pub fn new(data: R) -> Result<WavDecoder<R>, hound::Error> {
        let reached_eof = Arc::new(AtomicBool::new(false));
        let reader = WavReader::new(EofTracker {
            inner: data,
            reached_eof: reached_eof.clone(),
        })?;
        let spec = reader.spec();
        if !is_supported(spec.sample_format, spec.bits_per_sample) {
            return Err(hound::Error::Unsupported);
        }
        if spec.sample_rate == 0 {
            return Err(hound::Error::FormatError("sample rate is 0"));
        }

        let sample_rate = spec.sample_rate;
        let channel_count = spec.channels;

        Ok(WavDecoder {
            reader,
            reached_eof,
            sample_rate,
            channel_count,
        })
    }

    /// Return the wrapped Reader
    pub fn into_inner(self) -> R {
        self.reader.into_inner().inner
    }
}

/// Remembers if the end of `inner` was reached. hound returns an error of kind
/// Other instead of UnexpectedEof if data ends part way through so this is used
/// to know that a read error was caused by reaching the end.
struct EofTracker<R> {
    inner: R,
    reached_eof: Arc<AtomicBool>,
}

impl<R: Read> Read for EofTracker<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n == 0 && !buf.is_empty() {
            self.reached_eof.store(true, Ordering::Relaxed);
        }
        Ok(n)
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
        let mut filled = match (spec.sample_format, spec.bits_per_sample) {
            (SampleFormat::Float, 32) => fill(self.reader.samples(), buf, ch, f32_to_i16),
            (SampleFormat::Int, 8) => fill(self.reader.samples(), buf, ch, i8_to_i16),
            (SampleFormat::Int, 16) => fill(self.reader.samples(), buf, ch, |s: i16| s),
            (SampleFormat::Int, 24) => fill(self.reader.samples(), buf, ch, i24_to_i16),
            (SampleFormat::Int, 32) => fill(self.reader.samples(), buf, ch, i32_to_i16),
            // Checked in new
            _ => Filled::stopped(0, Stop::Error(hound::Error::Unsupported.into())),
        };
        if matches!(filled.stop, Some(Stop::Error(_))) && self.reached_eof.load(Ordering::Relaxed) {
            // The data is shorter than the header claims. Treat it as the end.
            filled.stop = Some(Stop::Finished);
        }
        filled
    }

    fn on_start_of_batch(&mut self) {}
}

/// Whether next_samples can convert samples of this format to i16.
fn is_supported(sample_format: SampleFormat, bits_per_sample: u16) -> bool {
    matches!(
        (sample_format, bits_per_sample),
        (SampleFormat::Float, 32)
            | (SampleFormat::Int, 8)
            | (SampleFormat::Int, 16)
            | (SampleFormat::Int, 24)
            | (SampleFormat::Int, 32)
    )
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
