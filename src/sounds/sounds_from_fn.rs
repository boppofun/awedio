use crate::{Filled, Sound, Stop};

type SoundGenerator = Box<dyn FnMut() -> Option<Box<dyn Sound>> + Send>;

/// Play sounds produced by a function returning sounds one after the other.
///
/// The generator function is called after each previously produced sound has
/// returned finished. After `SoundsFromFn` returns None
/// this sound returns Finished. If an Error is returned from the current sound
/// that sound is dropped and the Error is returned. The generator is called
/// again to produce the sound for the next call of next_samples.
///
/// This can be used to create sounds that loop forever without storing all
/// samples in memory.
///
/// If more than 8 sounds in a row finish without producing any samples
/// (e.g. looping an empty file) this sound returns Finished and `generator`
/// will no longer be called. This avoids calling `generator` forever.
pub struct SoundsFromFn {
    generator: SoundGenerator,
    current: Option<Box<dyn Sound>>,
    current_channel_count: u16,
    current_sample_rate: u32,
    /// The number of sounds in a row that have finished without producing any
    /// samples.
    num_empty_in_a_row: u32,
}

/// The maximum value of `num_empty_in_a_row` before finishing.
const MAX_EMPTY_IN_A_ROW: u32 = 8;

impl SoundsFromFn {
    /// Call `generator` to generate Sounds that will be played to completion.
    /// If `generator` returns None, this Sound will be Finished and `generator`
    /// will no longer be called.
    ///
    /// ## Examples
    /// Play an audio file forever.
    ///
    /// ```rust
    /// # fn no_run() {
    /// use awedio::sounds::{SoundsFromFn, open_file};
    ///
    /// let generator = || Some(open_file("test.wav").unwrap());
    /// let forever_sound = SoundsFromFn::new(Box::new(generator));
    /// # }
    /// ```
    pub fn new(mut generator: SoundGenerator) -> Self {
        let current = generator();
        let mut to_return = Self {
            generator,
            current,
            current_channel_count: 0,
            current_sample_rate: 0,
            num_empty_in_a_row: 0,
        };
        to_return.update_metadata();
        to_return
    }

    fn update_metadata(&mut self) {
        self.current_channel_count = self.channel_count();
        self.current_sample_rate = self.sample_rate();
    }
}

impl Sound for SoundsFromFn {
    fn channel_count(&self) -> u16 {
        self.current
            .as_ref()
            .map(|s| s.channel_count())
            .unwrap_or(1)
    }

    fn sample_rate(&self) -> u32 {
        self.current
            .as_ref()
            .map(|s| s.sample_rate())
            .unwrap_or(1000)
    }

    fn on_start_of_batch(&mut self) {
        if let Some(current) = &mut self.current {
            current.on_start_of_batch();
        }
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let mut written = 0;
        loop {
            let Some(current) = &mut self.current else {
                return Filled::stopped(written, Stop::Finished);
            };
            let filled = current.next_samples(&mut buf[written..]);
            written += filled.written;
            if filled.written > 0 {
                self.num_empty_in_a_row = 0;
            }
            match filled.stop {
                None => return Filled::all(written),
                Some(Stop::MetadataChanged) => {
                    self.update_metadata();
                    return Filled::stopped(written, Stop::MetadataChanged);
                }
                Some(Stop::Paused) => return Filled::stopped(written, Stop::Paused),
                Some(Stop::Error(e)) => {
                    self.current = None;
                    self.current = (self.generator)();
                    self.update_metadata();
                    return Filled::stopped(written, Stop::Error(e));
                }
                Some(Stop::Finished) => {
                    if filled.written == 0 {
                        // The sound might have produced samples in an earlier
                        // call in which case it is counted as empty when it
                        // is not. That is fine since we only need to make
                        // sure this does not go on forever.
                        self.num_empty_in_a_row += 1;
                        if self.num_empty_in_a_row > MAX_EMPTY_IN_A_ROW {
                            self.current = None;
                            return Filled::stopped(written, Stop::Finished);
                        }
                    }
                    let old_channel_count = self.current_channel_count;
                    let old_sample_rate = self.current_sample_rate;
                    self.current = None;
                    self.current = (self.generator)();
                    self.update_metadata();
                    if self.current.is_none() {
                        return Filled::stopped(written, Stop::Finished);
                    }
                    if old_sample_rate != self.current_sample_rate
                        || old_channel_count != self.current_channel_count
                    {
                        return Filled::stopped(written, Stop::MetadataChanged);
                    }
                    if written == buf.len() {
                        return Filled::all(written);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "./tests/sounds_from_fn.rs"]
mod tests;
