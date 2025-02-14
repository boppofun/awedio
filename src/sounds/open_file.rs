use crate::Sound;
use std::{fs::File, io::BufReader};

/// Create a Sound that reads from a file with the correct decoder based on the
/// file extension.
///
/// If the file type is not able to be decoded than an
/// [std::io::ErrorKind::Unsupported] is returned.
///
/// Uses a BufReader internally with the default capacity.
///
/// The returned Sound reads using File. This is generally not recommended
/// on the renderer thread as reading from a file could block the renderer.
/// Consider convert the sound to a memory_sound which is stored entirely in RAM
/// (and can be cloned cheaply).
pub fn open_file<P: AsRef<std::path::Path>>(path: P) -> Result<Box<dyn Sound>, crate::Error> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    open_file_with_reader(path.as_ref(), reader)
}

/// Same as `open_file` but with an explicit BufReader capacity.
pub fn open_file_with_buffer_capacity<P: AsRef<std::path::Path>>(
    path: P,
    buffer_capacity: usize,
) -> Result<Box<dyn Sound>, crate::Error> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::with_capacity(buffer_capacity, file);
    open_file_with_reader(path.as_ref(), reader)
}

fn open_file_with_reader(
    path: &std::path::Path,
    reader: BufReader<File>,
) -> Result<Box<dyn Sound>, crate::Error> {
    let extension = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();
    let decoder: Box<dyn Sound> = match extension.as_ref() {
        #[cfg(feature = "rmp3-mp3")]
        "mp3" => Box::new(super::decoders::Mp3Decoder::new(reader)),
        #[cfg(feature = "qoa")]
        "qoa" => Box::new(super::decoders::QoaDecoder::new(reader)?),
        #[cfg(feature = "hound-wav")]
        "wav" => Box::new(super::decoders::WavDecoder::new(reader)?),
        "_SILENCE_NEVER_MATCH_" => {
            println!(
                "Included to satisfy unused warnings when all features are off: {:?}",
                reader
            );
            Box::new(crate::sounds::Silence::new(1, 1000))
        }
        #[cfg(feature = "symphonia")]
        _ => Box::new(super::decoders::SymphoniaDecoder::new(
            Box::new(reader.into_inner()),
            Some(&extension),
        )?),
        #[cfg(not(feature = "symphonia"))]
        _ => return Err(std::io::Error::from(std::io::ErrorKind::Unsupported).into()),
    };
    Ok(decoder)
}
