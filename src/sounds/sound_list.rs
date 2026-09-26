use crate::sounds::wrappers::{AddSound, ClearSounds};
use crate::{Filled, Sound, Stop};

/// Play Sounds sequentially one after the other.
///
/// Only after a Sound has returned `Stop::Finished` will the next Sound
/// start playing.
///
/// If an Error is returned from a Sound it is dropped and the error is
/// propagated to the caller. Calling next_samples again would continue
/// with the next Sound in the list.
pub struct SoundList {
    sounds: Vec<Box<dyn Sound>>,
    was_empty: bool,
}

impl SoundList {
    /// Create a new empty SoundList.
    pub fn new() -> Self {
        SoundList {
            sounds: Vec::new(),
            was_empty: false,
        }
    }

    /// Add a Sound to be played after any existing sounds have `Finished`.
    pub fn add(&mut self, sound: Box<dyn Sound>) {
        if self.sounds.is_empty() {
            self.was_empty = true;
        }
        self.sounds.push(sound);
    }

    /// Inserts a sound at position `index`, shifting all elements after it to
    /// the right.
    ///
    /// Panics
    ///
    /// Panics if `index > len`.
    pub fn insert(&mut self, index: usize, sound: Box<dyn Sound>) {
        if self.sounds.is_empty() {
            self.was_empty = true;
        }
        self.sounds.insert(index, sound)
    }

    /// Stop all sounds including the currently playing one.
    pub fn clear(&mut self) {
        self.sounds.clear();
    }

    /// Returns the number of sounds currently in the list.
    pub fn len(&self) -> usize {
        self.sounds.len()
    }

    /// Returns `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.sounds.is_empty()
    }
}

impl From<Vec<Box<dyn Sound>>> for SoundList {
    fn from(sounds: Vec<Box<dyn Sound>>) -> Self {
        let was_empty = sounds.is_empty();
        SoundList { sounds, was_empty }
    }
}

impl From<SoundList> for Vec<Box<dyn Sound>> {
    fn from(list: SoundList) -> Self {
        list.sounds
    }
}

impl FromIterator<Box<dyn Sound>> for SoundList {
    fn from_iter<T: IntoIterator<Item = Box<dyn Sound>>>(iter: T) -> Self {
        let vec: Vec<_> = iter.into_iter().collect();
        vec.into()
    }
}

// Returned only when no sounds exist so they shouldn't be used in practice.
const DEFAULT_CHANNEL_COUNT: u16 = 2;
const DEFAULT_SAMPLE_RATE: u32 = 48000;

impl Sound for SoundList {
    fn channel_count(&self) -> u16 {
        self.sounds
            .first()
            .map(|s| s.channel_count())
            .unwrap_or(DEFAULT_CHANNEL_COUNT)
    }

    fn sample_rate(&self) -> u32 {
        self.sounds
            .first()
            .map(|s| s.sample_rate())
            .unwrap_or(DEFAULT_SAMPLE_RATE)
    }

    fn on_start_of_batch(&mut self) {
        for sound in &mut self.sounds {
            sound.on_start_of_batch();
        }
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        if self.sounds.is_empty() {
            return Filled::stopped(0, Stop::Finished);
        }
        if self.was_empty {
            self.was_empty = false;
            return Filled::stopped(0, Stop::MetadataChanged);
        }
        let mut written = 0;
        loop {
            let Some(next_sound) = self.sounds.first_mut() else {
                return Filled::stopped(written, Stop::Finished);
            };
            let filled = next_sound.next_samples(&mut buf[written..]);
            written += filled.written;
            match filled.stop {
                None => return Filled::all(written),
                Some(Stop::MetadataChanged) | Some(Stop::Paused) => {
                    return Filled {
                        written,
                        stop: filled.stop,
                    }
                }
                Some(Stop::Error(e)) => {
                    self.sounds.remove(0);
                    return Filled::stopped(written, Stop::Error(e));
                }
                Some(Stop::Finished) => {
                    let channel_count = next_sound.channel_count();
                    let sample_rate = next_sound.sample_rate();
                    self.sounds.remove(0);
                    let Some(next) = self.sounds.first() else {
                        return Filled::stopped(written, Stop::Finished);
                    };
                    if next.channel_count() != channel_count || next.sample_rate() != sample_rate {
                        // The next sound has different metadata. Instead of
                        // normalizing here let downstream normalize.
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

impl AddSound for SoundList {
    fn add(&mut self, sound: Box<dyn Sound>) {
        SoundList::add(self, sound);
    }
}

impl ClearSounds for SoundList {
    fn clear(&mut self) {
        self.clear();
    }
}

impl Default for SoundList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SoundList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundList")
            .field("sounds", &format!("{} sounds", self.sounds.len()))
            .field("was_empty", &self.was_empty)
            .finish()
    }
}

#[cfg(test)]
#[path = "./tests/sound_list.rs"]
mod tests;
