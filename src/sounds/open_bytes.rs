use crate::Sound;

/// Create a Sound from bytes. This is best used with [std::include_bytes!], as the bytes must have
/// a static lifetime.
///
/// It tries to autodetect the file type
///
/// If the file type is not able to be decoded then an
/// [std::io::ErrorKind::Unsupported] is returned.
///
/// Like [crate::sounds::open_file], you may want to use it in conjunction with
/// [crate::sounds::memory_sound] to be cheaper.
pub fn open_bytes(bytes: &'static [u8]) -> Result<Box<dyn Sound>, crate::Error> {
    use std::io::Cursor;

    // underscore here so we don't get warnings with the features
    #[cfg(feature = "rmp3-mp3")]
    let _error = match super::decoders::Mp3Decoder::new(bytes) {
        Err(e) => e,
        Ok(decoder) => return Ok(Box::new(decoder)),
    };

    #[cfg(feature = "qoa")]
    let _error = match super::decoders::QoaDecoder::new(bytes) {
        Err(e) => e,
        Ok(decoder) => return Ok(Box::new(decoder)),
    };

    #[cfg(feature = "hound-wav")]
    let _error = match super::decoders::WavDecoder::new(bytes) {
        Err(e) => e,
        Ok(decoder) => return Ok(Box::new(decoder)),
    };

    #[cfg(feature = "symphonia")]
    let _error = match super::decoders::SymphoniaDecoder::new(Box::new(Cursor::new(bytes)), None) {
        Err(e) => e,
        Ok(decoder) => return Ok(Box::new(decoder)),
    };
    Err(_error.into())
}
