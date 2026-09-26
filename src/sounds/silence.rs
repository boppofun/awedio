/// A forever stream of samples of value 0 (creating no sound).
pub struct Silence {
    channel_count: u16,
    sample_rate: u32,
}

impl Silence {
    /// Create a new sound that will return 0 samples forever and
    /// return the specified channel count and sample rate from
    /// their respective methods.
    pub fn new(channel_count: u16, sample_rate: u32) -> Silence {
        Silence {
            channel_count,
            sample_rate,
        }
    }
}

impl crate::Sound for Silence {
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> crate::Filled {
        buf.fill(0);
        crate::Filled::all(buf.len())
    }

    fn on_start_of_batch(&mut self) {}
}
