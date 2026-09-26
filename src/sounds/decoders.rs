//! Decoders for various audio formats and file types.
//!
//! These are normally accessed via
//! [sounds::open_file][crate::sounds::open_file()].
#[cfg(feature = "mbop3-mp3")]
mod mbop3;
#[cfg(feature = "rmp3-mp3")]
mod mp3;
#[cfg(feature = "qoa")]
mod qoa;
#[cfg(feature = "symphonia")]
mod symphonia;
#[cfg(feature = "hound-wav")]
mod wav;

#[cfg(feature = "mbop3-mp3")]
pub use mbop3::Mbop3Decoder;
#[cfg(feature = "rmp3-mp3")]
pub use mp3::Mp3Decoder;
#[cfg(feature = "qoa")]
pub use qoa::QoaDecoder;
#[cfg(feature = "qoa")]
pub use qoaudio::DecodeError as QoaDecodeError;
#[cfg(feature = "symphonia")]
pub use symphonia::SymphoniaDecoder;
#[cfg(feature = "hound-wav")]
pub use wav::WavDecoder;
