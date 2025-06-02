use crate::sounds::wrappers::Controllable;
use crate::sounds::wrappers::Wrapper;
use crate::sounds::SoundMixer;
use crate::NextSample;
use crate::Sound;

use super::backend_source::BackendSource;

/// The default [BackendSource]. Renderer is essentially half of
/// [Manager][crate::manager::Manager].
pub struct Renderer {
    mixer: Controllable<SoundMixer>,
}

impl Renderer {
    pub(crate) fn new(mixer: Controllable<SoundMixer>) -> Self {
        Renderer { mixer }
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

    /// Get the next sample.
    /// `MetadataChanged` will only be returned from Renderer if
    /// `set_output_channel_count_and_sample_rate` was called. If `Paused`
    /// is returned the backend may choose to pause itself or play silence.
    /// `Finished` will be returned if no sounds are playing and the Manager of
    /// the Renderer has been dropped.
    ///
    /// Guaranteed to not return an Error.
    fn next_sample(&mut self) -> Result<NextSample, crate::Error> {
        self.mixer.next_sample()
    }

    /// Inform the playing or queued sounds that a new batch of samples will be
    /// requested. This must only be called when the next sample to be delivered
    /// from `next_sample` is for the first channel.
    ///
    /// See [Sound::on_start_of_batch]
    fn on_start_of_batch(&mut self) {
        self.mixer.on_start_of_batch()
    }
}
