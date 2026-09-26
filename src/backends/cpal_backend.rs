//! [`CpalBackend`] outputs audio using the [cpal](https://www.docs.rs/cpal)
//! crate.

use crate::{
    manager::{BackendSource, Manager, Renderer},
    Sound,
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Error as CpalError, ErrorKind, FromSample, SizedSample,
};
use std::error::Error;

pub use cpal::BufferSize as CpalBufferSize;

/// A backend that uses [cpal](https://www.docs.rs/cpal) to output to devices.
///
/// This backend does not currently update the output device if the default
/// output device of the host changes.
pub struct CpalBackend {
    channel_count: u16,
    sample_rate: u32,
    sample_format: cpal::SampleFormat,
    buffer_size: CpalBufferSize,
    device: cpal::Device,
    stream: Option<cpal::Stream>,
}

impl CpalBackend {
    /// Create a new CpalBackend with defaults for all fields.
    ///
    /// Returns None if a default device or config could not be obtained.
    pub fn with_defaults() -> Option<CpalBackend> {
        let host = cpal::default_host();

        let device = host.default_output_device()?;

        let default_config = device.default_output_config().ok()?;
        let sample_rate = default_config.sample_rate();
        let channel_count = default_config.channels();
        let sample_format = default_config.sample_format();

        Some(CpalBackend {
            channel_count,
            sample_rate,
            buffer_size: CpalBufferSize::Default,
            device,
            stream: None,
            sample_format,
        })
    }

    /// Create a new backend.
    ///
    /// Returns None if an output device is not found
    pub fn with_default_host_and_device(
        channel_count: u16,
        sample_rate: u32,
        buffer_size: CpalBufferSize,
    ) -> Option<CpalBackend> {
        let host = cpal::default_host();

        let device = host.default_output_device()?;
        let sample_format = device.default_output_config().ok()?.sample_format();

        Some(CpalBackend {
            channel_count,
            sample_rate,
            buffer_size,
            device,
            stream: None,
            sample_format,
        })
    }

    /// Create a new CpalBackend specifying all fields.
    pub fn new(
        channel_count: u16,
        sample_rate: u32,
        buffer_size: CpalBufferSize,
        device: cpal::Device,
        sample_format: cpal::SampleFormat,
    ) -> CpalBackend {
        CpalBackend {
            channel_count,
            sample_rate,
            buffer_size,
            device,
            stream: None,
            sample_format,
        }
    }
}

impl CpalBackend {
    /// Start a cpal output stream and connect it to the returned Manager.
    ///
    /// Only a single stream is supported at a time per CpalBackend object.
    ///
    /// Cpal stream errors will be reported by calling `error_callback`.
    pub fn start<E>(&mut self, error_callback: E) -> Result<Manager, CpalBackendError>
    where
        E: FnMut(CpalError) + Send + 'static,
    {
        let (manager, mut renderer) = Manager::new();
        renderer.set_output_channel_count_and_sample_rate(self.channel_count, self.sample_rate);
        let Some(crate::Stop::MetadataChanged) = renderer.next_samples(&mut []).stop else {
            panic!("expected MetadataChanged event")
        };

        let config = cpal::StreamConfig {
            channels: self.channel_count,
            sample_rate: self.sample_rate,
            buffer_size: self.buffer_size,
        };

        let timeout = None;
        let stream = match self.sample_format {
            cpal::SampleFormat::I16 => self
                .device
                .build_output_stream(
                    config,
                    make_data_callback::<i16>(renderer, self.channel_count),
                    error_callback,
                    timeout,
                )
                .map_err(CpalBackendError::BuildStream)?,
            cpal::SampleFormat::F32 => self
                .device
                .build_output_stream(
                    config,
                    make_data_callback::<f32>(renderer, self.channel_count),
                    error_callback,
                    timeout,
                )
                .map_err(CpalBackendError::BuildStream)?,
            sample_format => {
                return Err(CpalBackendError::BuildStream(CpalError::with_message(
                    ErrorKind::UnsupportedConfig,
                    format!(
                        "unsupported output stream sample format: {:?}",
                        sample_format
                    ),
                )))
            }
        };

        stream.play().map_err(CpalBackendError::PlayStream)?;
        self.stream = Some(stream);
        Ok(manager)
    }
}

/// Converts Awedio's internal i16 samples to the format required by the audio
/// device (type T).
fn make_data_callback<T>(
    mut renderer: Renderer,
    channel_count: u16,
) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo)
where
    T: SizedSample + FromSample<i16>,
{
    // The renderer is called with chunks of this many frames. It is not based
    // on the cpal buffer size since that is not known with
    // CpalBufferSize::Default and resizing in the callback would allocate on
    // the audio thread. Per call overhead is negligible past ~128 frames and
    // this matches the chunk size the SoundMixer renders in internally.
    const CHUNK_NUM_FRAMES: usize = 256;
    let mut samples = vec![0_i16; CHUNK_NUM_FRAMES * channel_count as usize];
    move |buffer: &mut [T], _info: &cpal::OutputCallbackInfo| {
        assert!(buffer.len().is_multiple_of(channel_count as usize));

        renderer.on_start_of_batch();

        let mut stopped = false;
        for out in buffer.chunks_mut(samples.len()) {
            let samples = &mut samples[..out.len()];
            let written = if stopped {
                0
            } else {
                let filled = renderer.next_samples(samples);
                match filled.stop {
                    None => (),
                    Some(crate::Stop::MetadataChanged) => {
                        unreachable!("we never change metadata mid-batch")
                    }
                    // TODO: implement pausing and finishing
                    Some(crate::Stop::Paused) | Some(crate::Stop::Finished) => stopped = true,
                    Some(crate::Stop::Error(_)) => {
                        unreachable!("renderer should never return an Error")
                    }
                }
                filled.written
            };
            samples[written..].fill(0);
            for (o, s) in out.iter_mut().zip(samples.iter()) {
                *o = T::from_sample(*s);
            }
        }
    }
}

/// An error from the [`CpalBackend`]
#[derive(Debug)]
pub enum CpalBackendError {
    /// No output device or configuration found.
    NoDevice,
    /// An error while building the output stream
    BuildStream(CpalError),
    /// An error while starting to play the stream.
    PlayStream(CpalError),
}

impl std::fmt::Display for CpalBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CpalBackendError::NoDevice => {
                write!(f, "unable to find suitable device or config")
            }
            CpalBackendError::BuildStream(_) => {
                write!(f, "unable to build stream")
            }
            CpalBackendError::PlayStream(_) => {
                write!(f, "unable to play stream")
            }
        }
    }
}

impl Error for CpalBackendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CpalBackendError::NoDevice => None,
            CpalBackendError::BuildStream(e) => Some(e),
            CpalBackendError::PlayStream(e) => Some(e),
        }
    }
}
