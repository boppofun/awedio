use std::f32::consts::TAU;

/// A constant pitch sound of infinite length.
pub struct SineWave {
    freq: f32,
    sample_rate: u32,
    sample_num: u32,
    reset_num: u32,
}

impl SineWave {
    /// A constant pitch sound with a default sample rate of 48,000.
    pub fn new(freq: f32) -> SineWave {
        Self::with_sample_rate(freq, 48000)
    }

    /// A constant pitch sound with `sample_rate`.
    pub fn with_sample_rate(freq: f32, sample_rate: u32) -> SineWave {
        let reset_num = find_reset_num(freq, sample_rate);

        SineWave {
            freq,
            sample_rate,
            sample_num: 0,
            reset_num,
        }
    }
}

/// find a sample number where we can reset to 0 where the value will
/// be close to 0 to minimize distortion when resetting.
///
/// We want to minimize the sample number though because large numbers
/// cause distortions
fn find_reset_num(freq: f32, sample_rate: u32) -> u32 {
    let mut best_error = f64::MAX;
    let mut best_reset_num = 0;

    for multiple in 1..100 {
        let reset_num = sample_rate as f64 * (multiple as f64) / freq as f64;
        let error = (reset_num.round() - reset_num).abs();
        if error < best_error {
            best_error = error;
            best_reset_num = reset_num.round() as u32;
            if error == 0.0 {
                break;
            }
        }
    }

    best_reset_num - 1
}

impl crate::Sound for SineWave {
    fn channel_count(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_sample(&mut self) -> Result<crate::NextSample, crate::Error> {
        if self.sample_num == self.reset_num {
            self.sample_num = 0;
        } else {
            self.sample_num += 1;
        }
        let value = self.sample_num as f32 * self.freq * TAU / self.sample_rate as f32;
        let sample = (value.sin() * i16::MAX as f32) as i16;
        Ok(crate::NextSample::Sample(sample))
    }

    fn on_start_of_batch(&mut self) {}
}

#[cfg(test)]
#[path = "./tests/sine_wave.rs"]
mod tests;
