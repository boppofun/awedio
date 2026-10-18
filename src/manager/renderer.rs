use crate::sounds::wrappers::AddSound;
use crate::sounds::wrappers::ClearSounds;
use crate::sounds::wrappers::Controllable;
use crate::sounds::wrappers::Wrapper;
use crate::sounds::SoundMixer;
use crate::Filled;
use crate::Sound;

use super::backend_source::BackendSource;

/// The default [BackendSource]. Renderer is essentially half of
/// [Manager][crate::manager::Manager].
pub struct Renderer {
    mixer: Controllable<RootMixer>,
}

impl Renderer {
    pub(crate) fn new(mixer: Controllable<RootMixer>) -> Self {
        Renderer { mixer }
    }
}

/// The mixer of all played sounds and the Manager's start of batch callback.
pub(crate) struct RootMixer {
    mixer: SoundMixer,
    pub(crate) on_start_of_batch: Option<Box<dyn FnMut() + Send>>,
}

impl RootMixer {
    pub(crate) fn new(mixer: SoundMixer) -> Self {
        RootMixer {
            mixer,
            on_start_of_batch: None,
        }
    }
}

impl Sound for RootMixer {
    fn channel_count(&self) -> u16 {
        self.mixer.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.mixer.sample_rate()
    }

    #[inline]
    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        self.mixer.next_samples(buf)
    }

    fn on_start_of_batch(&mut self) {
        if let Some(f) = &mut self.on_start_of_batch {
            f();
        }
        self.mixer.on_start_of_batch()
    }
}

impl AddSound for RootMixer {
    fn add(&mut self, sound: Box<dyn Sound>) {
        self.mixer.add(sound)
    }
}

impl ClearSounds for RootMixer {
    fn clear(&mut self) {
        self.mixer.clear()
    }
}

impl BackendSource for Renderer {
    fn set_output_channel_count_and_sample_rate(
        &mut self,
        output_channel_count: u16,
        output_sample_rate: u32,
    ) {
        self.mixer
            .inner_mut()
            .mixer
            .set_output_channel_count_and_sample_rate(output_channel_count, output_sample_rate);
    }
}

impl Sound for Renderer {
    fn channel_count(&self) -> u16 {
        self.mixer.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.mixer.sample_rate()
    }

    /// Get the next samples.
    /// `MetadataChanged` will only be returned from Renderer if
    /// `set_output_channel_count_and_sample_rate` was called. If `Paused`
    /// is returned the backend may choose to pause itself or play silence.
    /// `Finished` will be returned if no sounds are playing and the Manager of
    /// the Renderer has been dropped.
    ///
    /// Guaranteed to not return an Error.
    #[inline]
    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        self.mixer.next_samples(buf)
    }

    /// Inform the playing or queued sounds that a new batch of samples will be
    /// requested.
    ///
    /// See [Sound::on_start_of_batch]
    fn on_start_of_batch(&mut self) {
        self.mixer.on_start_of_batch()
    }
}
