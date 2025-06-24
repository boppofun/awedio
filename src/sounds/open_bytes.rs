use crate::Sound;
use std::io::Read;
#[cfg(feature = "symphonia")]
use symphonia::core::io::MediaSource;

/// Create a Sound from bytes. This is best used with [std::include_bytes!], as the bytes must have
/// a static lifetime.
///
/// It tries to autodetect the file type.
pub fn open_bytes<T: AsRef<[u8]> + std::marker::Sync>(
    bytes: &'static T,
) -> Result<Box<dyn Sound>, crate::Error> {
    use std::io::Cursor;
    open_reader(Cursor::new(bytes))
}

/// Create a Sound from a reader.
/// It tries to autodetect the file type.
pub fn open_reader<R: Read + Send + MediaSource + 'static>(
    mut reader: R,
) -> Result<Box<dyn Sound>, crate::Error> {
    let _error: crate::Error = std::io::Error::from(std::io::ErrorKind::Unsupported).into();

    #[cfg(feature = "rmp3-mp3")]
    // This is commented out because MP3Decoder doesn't
    // return a Result. For now, in that case we just fail

    // let __error = match super::decoders::Mp3Decoder::new(reader.by_ref()) {
    //     Err(e) => e,
    //     Ok(decoder) => return Ok(Box::new(decoder)),
    // };

    // underscore here so we don't get warnings with the features
    let _error: crate::Error = std::io::Error::from(std::io::ErrorKind::Unsupported).into();

    #[cfg(feature = "qoa")]
    // Seems to be a little messy but is the best I could come up
    // with so we could reuse reader if there are multiple features
    // at once
    let _error = match super::decoders::QoaDecoder::new(reader.by_ref()) {
        Err(e) => e,
        Ok(_) => return Ok(Box::new(super::decoders::QoaDecoder::new(reader).unwrap())),
    };

    #[cfg(feature = "hound-wav")]
    let _error = match super::decoders::WavDecoder::new(reader.by_ref()) {
        Err(e) => e,
        Ok(_) => return Ok(Box::new(super::decoders::WavDecoder::new(reader).unwrap())),
    };

    #[cfg(feature = "symphonia")]
    let _error = match super::decoders::SymphoniaDecoder::new(Box::new(reader), None) {
        Err(e) => e,
        Ok(decoder) => return Ok(Box::new(decoder)),
    };
    Err(_error.into())
}
