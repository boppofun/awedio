use crate::{Filled, Sound, Stop};

use super::Wrapper;

// Originally forked from https://github.com/RustAudio/rodio/blob/d5b9ae3467dab4316ee77b260a5b7432f74866b0/src/conversions/sample_rate.rs

/// Convert a Sound from one sample rate (number of samples per second) to
/// another.
///
/// Conversion is done with linear interpolation. If the inner sound already
/// has the output sample rate, samples are passed through unchanged.
pub struct SampleRateConverter<S: Sound> {
    /// The from Sound we are pulling samples from.
    inner: S,
    /// The output sample rate in samples per second.
    to_rate: u32,
    /// The channel count of the samples we output. This can differ from
    /// `inner.channel_count()` until we have processed the MetadataChanged of
    /// inner.
    channel_count: u16,
    /// Whether samples are passed through unchanged or converted.
    pass_through: bool,
    /// This is not the samples per second of the output but a possibly scaled
    /// down value.
    to_rate_scaled: u32,
    /// This is not the samples per second of inner but a possibly scaled down
    /// value.
    from_rate_scaled: u32,
    /// The position of the next output frame between `current_frame` and
    /// `next_frame` in units of 1/`to_rate_scaled` of an input frame. Always
    /// less than `to_rate_scaled`.
    phase: u32,
    /// The number of input frames to drop before the next output frame.
    pending_advance: u32,
    /// One sample per channel, extracted from `inner`.
    current_frame: Vec<i16>,
    /// The frame right after `current_frame`.
    next_frame: Vec<i16>,
    /// How many of `current_frame` and `next_frame` hold valid samples (0, 1,
    /// or 2).
    frames_loaded: u8,
    /// Samples read from `inner` but not yet used.
    input: Vec<i16>,
    /// The next unused sample in input.
    input_pos: usize,
    /// The number of valid samples in input.
    input_len: usize,
    /// A stop returned from inner while filling `input`. It is handled once
    /// all samples in `input` have been used.
    pending_stop: Option<Stop>,
    /// Inner has stopped (other than paused) and `next_frame` is a copy of
    /// `current_frame` so the output frames of the last input frame are
    /// produced before the stop is returned.
    flushing: bool,
}

/// The maximum number of frames read from inner at a time.
const MAX_INPUT_FRAMES: u64 = 256;

impl<S> SampleRateConverter<S>
where
    S: Sound,
{
    /// Create a new SampleRateConverter with an output sample rate of
    /// `to_rate`.
    pub fn new(inner: S, to_rate: u32) -> SampleRateConverter<S> {
        assert!(to_rate >= 1);
        let mut new = SampleRateConverter {
            channel_count: inner.channel_count(),
            inner,
            to_rate,
            pass_through: true,
            to_rate_scaled: 1,
            from_rate_scaled: 1,
            phase: 0,
            pending_advance: 0,
            current_frame: Vec::new(),
            next_frame: Vec::new(),
            frames_loaded: 0,
            input: Vec::new(),
            input_pos: 0,
            input_len: 0,
            pending_stop: None,
            flushing: false,
        };
        new.init();
        new
    }

    /// (Re)initialize the conversion using the current metadata of inner. Any
    /// buffered samples are dropped.
    fn init(&mut self) {
        let channel_count = self.inner.channel_count();
        let from_rate = self.inner.sample_rate();
        assert!(from_rate >= 1);

        fn gcd(a: u32, b: u32) -> u32 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }
        let gcd = gcd(from_rate, self.to_rate);

        self.channel_count = channel_count;
        self.pass_through = from_rate == self.to_rate;
        self.to_rate_scaled = self.to_rate / gcd;
        self.from_rate_scaled = from_rate / gcd;
        self.phase = 0;
        self.pending_advance = 0;
        self.current_frame.clear();
        self.current_frame.resize(channel_count as usize, 0);
        self.next_frame.clear();
        self.next_frame.resize(channel_count as usize, 0);
        self.frames_loaded = 0;
        self.input_pos = 0;
        self.input_len = 0;
        self.flushing = false;
    }

    /// Unwrap the inner Sound. Any samples buffered by the converter are
    /// lost.
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Load the next frame from `input` into `current_frame` or `next_frame`.
    ///
    /// Returns false if no input is available.
    #[inline]
    fn load_frame_from_input(&mut self) -> bool {
        let channel_count = self.channel_count as usize;
        if self.input_pos + channel_count > self.input_len {
            return false;
        }
        let frame = &self.input[self.input_pos..self.input_pos + channel_count];
        self.input_pos += channel_count;
        if self.frames_loaded == 0 {
            self.current_frame.copy_from_slice(frame);
        } else {
            debug_assert_eq!(self.frames_loaded, 1);
            self.next_frame.copy_from_slice(frame);
        }
        self.frames_loaded += 1;
        true
    }

    /// Read more samples from inner into `input`. `output_frames_wanted` is
    /// used to estimate how many input frames are needed.
    fn refill_input(&mut self, output_frames_wanted: usize) {
        debug_assert!(self.pending_stop.is_none());
        let channel_count = self.channel_count as usize;
        let frames = (output_frames_wanted as u64 * self.from_rate_scaled as u64
            / self.to_rate_scaled as u64
            + 2)
        .min(MAX_INPUT_FRAMES) as usize;
        let len = frames * channel_count;
        if self.input.len() < len {
            self.input.resize(len, 0);
        }
        let filled = self.inner.next_samples(&mut self.input[..len]);
        self.input_pos = 0;
        self.input_len = filled.written;
        self.pending_stop = filled.stop;
    }

    fn convert(&mut self, buf: &mut [i16]) -> Filled {
        let channel_count = self.channel_count as usize;
        let mut written = 0;
        while written < buf.len() {
            // Get current_frame and next_frame loaded for the next output
            // frame.
            while self.pending_advance > 0 || self.frames_loaded < 2 {
                if self.pending_advance > 0 && self.frames_loaded > 0 {
                    if self.frames_loaded == 2 {
                        std::mem::swap(&mut self.current_frame, &mut self.next_frame);
                    }
                    self.frames_loaded -= 1;
                    self.pending_advance -= 1;
                    continue;
                }
                if self.load_frame_from_input() {
                    continue;
                }
                match self.pending_stop.take() {
                    None => self.refill_input((buf.len() - written) / channel_count),
                    Some(Stop::Paused) => {
                        // Keep our state so we continue seamlessly when inner
                        // has more samples.
                        return Filled::stopped(written, Stop::Paused);
                    }
                    Some(stop) if self.frames_loaded == 1 && !self.flushing => {
                        // There is no frame after the last input frame to
                        // interpolate with. Hold the last frame so its output
                        // frames are not lost.
                        self.next_frame.copy_from_slice(&self.current_frame);
                        self.frames_loaded = 2;
                        self.flushing = true;
                        self.pending_stop = Some(stop);
                    }
                    Some(Stop::MetadataChanged) => {
                        self.init();
                        return Filled::stopped(written, Stop::MetadataChanged);
                    }
                    Some(stop) => {
                        // Start over when more samples come (e.g. after a
                        // recoverable error).
                        self.frames_loaded = 0;
                        self.flushing = false;
                        self.phase = 0;
                        self.pending_advance = 0;
                        return Filled::stopped(written, stop);
                    }
                }
            }

            // The output frame is a linear interpolation between
            // current_frame and next_frame. `fraction` is in Q15.
            let fraction = ((self.phase as u64) << 15) / self.to_rate_scaled as u64;
            let fraction = fraction as i32;
            for ((out, cur), next) in buf[written..written + channel_count]
                .iter_mut()
                .zip(&self.current_frame)
                .zip(&self.next_frame)
            {
                *out = linear_interpolation(*cur, *next, fraction);
            }
            written += channel_count;

            self.phase += self.from_rate_scaled;
            self.pending_advance = self.phase / self.to_rate_scaled;
            self.phase %= self.to_rate_scaled;
        }
        Filled::all(written)
    }
}

impl<S> Sound for SampleRateConverter<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.to_rate
    }

    #[inline]
    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        if self.pass_through {
            let filled = self.inner.next_samples(buf);
            if let Some(Stop::MetadataChanged) = filled.stop {
                self.init();
            }
            return filled;
        }
        self.convert(buf)
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch()
    }
}

impl<S: Sound> Wrapper for SampleRateConverter<S> {
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

/// `fraction` is in Q15 and must be in the range [0, 1). The result is
/// rounded to the nearest integer with ties away from zero so rising and
/// falling signals are treated the same (i.e. no DC offset is introduced).
#[inline]
fn linear_interpolation(first: i16, second: i16, fraction: i32) -> i16 {
    // (second - first) fits in 17 bits and fraction in 15 bits so the product
    // plus the rounding term fits in an i32.
    const HALF: i32 = 1 << 14;
    let delta = second as i32 - first as i32;
    let product = delta * fraction;
    // Subtracting 1 for negative products rounds their ties away from zero.
    let rounded = (product + HALF - (product < 0) as i32) >> 15;
    (first as i32 + rounded) as i16
}

#[cfg(test)]
#[path = "./tests/sample_rate_converter.rs"]
mod tests;
