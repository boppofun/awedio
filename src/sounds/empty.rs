/// A Sound that immediately and always returns Finished
pub struct Empty {
    channel_count: u16,
    sample_rate: u32,
}

impl Empty {
    /// Create a new sound that will immediately and always returns Finished
    pub fn new(channel_count: u16, sample_rate: u32) -> Empty {
        Empty {
            channel_count,
            sample_rate,
        }
    }
}

impl crate::Sound for Empty {
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn next_samples(&mut self, _buf: &mut [i16]) -> crate::Filled {
        crate::Filled::stopped(0, crate::Stop::Finished)
    }

    fn on_start_of_batch(&mut self) {}
}
