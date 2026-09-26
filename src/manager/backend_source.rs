use crate::Sound;

/// Backends obtain the samples to output from a BackendSource. The default
/// BackendSource is the [Renderer](crate::manager::Renderer). You can wrap a
/// Renderer to make changes before output to a backend (e.g. a global volume or
/// mute control).
///
/// Different backends are implemented in external crates for different
/// platforms and use cases.
///
/// A BackendSource is a Sound with the following additional restrictions:
///
/// * The sound must output the sample rate and channel count given in the last
///   call to set_output_channel_count_and_sample_rate. The backend is required
///   to call set_output_channel_count_and_sample_rate before any samples are
///   pulled via next_samples and may call it again.
/// * The BackendSource is not allowed to return MetadataChanged unless a call
///   to set_output_channel_count_and_sample_rate has occurred since the last
///   MetadataChanged.
/// * BackendSource::next_samples is not allowed to return Stop::Error.
///
/// The backend is responsible for:
///
///  1. storing the BackendSource
///  2. calling BackendSource::set_output_channel_count_and_sample_rate()
///  3. periodically calling BackendSource::on_start_of_batch() followed by
///     one or more renderer.next_samples() calls (normally enough to fill some
///     number of milliseconds of an output buffer).
pub trait BackendSource: Sound {
    /// Set the output channel count and sample rate that the backend source
    /// should provide to the backend via calls to Sound::next_samples.
    fn set_output_channel_count_and_sample_rate(
        &mut self,
        output_channel_count: u16,
        output_sample_rate: u32,
    );
}
