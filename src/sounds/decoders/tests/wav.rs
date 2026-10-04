use super::*;
use crate::tests::BySample as _;
use crate::Stop;

const SINE_WAVE_FILE: &[u8] = include_bytes!("audiocheck.net_sin_1000Hz_0dBFS_0.1s.wav");

#[test]
fn samples_of_test_file() -> std::io::Result<()> {
    let mut decoder = WavDecoder::new(std::io::Cursor::new(SINE_WAVE_FILE))
        .unwrap()
        .by_sample();
    assert_eq!(decoder.sample_rate(), 44100);
    assert_eq!(decoder.channel_count(), 1);
    assert_eq!(decoder.next_sample().unwrap(), 0); // 1
    assert_eq!(decoder.next_sample().unwrap(), 4647); // 2
    assert_eq!(decoder.next_sample().unwrap(), 9201); // 3
    assert_eq!(decoder.next_sample().unwrap(), 13567); // 4
    assert_eq!(decoder.next_sample().unwrap(), 17659); // 5
    assert_eq!(decoder.next_sample().unwrap(), 21393); // 6
    assert_eq!(decoder.next_sample().unwrap(), 24693); // 7
    assert_eq!(decoder.next_sample().unwrap(), 27493); // 8
    assert_eq!(decoder.next_sample().unwrap(), 29736); // 9
    assert_eq!(decoder.next_sample().unwrap(), 31377); // 10
    assert_eq!(decoder.next_sample().unwrap(), 32381); // 11
    assert_eq!(decoder.next_sample().unwrap(), 32729); // 12
    assert_eq!(decoder.next_sample().unwrap(), 32414); // 13
    for _i in 0..4398 {
        // println!("i: {_i}");
        decoder.next_sample().unwrap();
    }
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    Ok(())
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || Box::new(WavDecoder::new(std::io::Cursor::new(SINE_WAVE_FILE)).unwrap()),
        10000,
    );
}

/// Build a WAV file with a WAVE_FORMAT_EXTENSIBLE fmt chunk and `data`.
fn extensible_wav(
    channels: u16,
    sample_rate: u32,
    container_bits: u16,
    valid_bits: u16,
    float: bool,
    data: &[u8],
) -> Vec<u8> {
    let block_align = channels * container_bits / 8;
    let mut fmt = Vec::new();
    fmt.extend_from_slice(&0xfffe_u16.to_le_bytes());
    fmt.extend_from_slice(&channels.to_le_bytes());
    fmt.extend_from_slice(&sample_rate.to_le_bytes());
    fmt.extend_from_slice(&(sample_rate * block_align as u32).to_le_bytes());
    fmt.extend_from_slice(&block_align.to_le_bytes());
    fmt.extend_from_slice(&container_bits.to_le_bytes());
    fmt.extend_from_slice(&22_u16.to_le_bytes());
    fmt.extend_from_slice(&valid_bits.to_le_bytes());
    fmt.extend_from_slice(&0_u32.to_le_bytes());
    fmt.push(if float { 3 } else { 1 });
    fmt.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71,
    ]);
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&((4 + 8 + fmt.len() + 8 + data.len()) as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
    wav.extend_from_slice(&fmt);
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data.len() as u32).to_le_bytes());
    wav.extend_from_slice(data);
    wav
}

#[test]
fn supported_extensible() {
    let wav = extensible_wav(2, 8000, 16, 16, false, &[1, 0, 2, 0, 3, 0, 4, 0]);
    let mut decoder = WavDecoder::new(std::io::Cursor::new(wav)).unwrap();
    let mut buf = [0; 6];
    let filled = decoder.next_samples(&mut buf);
    assert_eq!(filled.written, 4);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert_eq!(&buf[..4], &[1, 2, 3, 4]);
}

#[test]
fn unsupported_bits_per_sample_is_error() {
    // Previously these panicked in next_samples.
    let wav = extensible_wav(1, 8000, 16, 12, false, &[0; 4]);
    assert!(WavDecoder::new(std::io::Cursor::new(wav)).is_err());
    let wav = extensible_wav(1, 8000, 64, 64, true, &[0; 16]);
    assert!(WavDecoder::new(std::io::Cursor::new(wav)).is_err());
}

#[test]
fn zero_sample_rate_is_error() {
    let wav = extensible_wav(1, 0, 16, 16, false, &[0; 4]);
    assert!(WavDecoder::new(std::io::Cursor::new(wav)).is_err());
}

#[test]
fn truncated_file_is_error() {
    // Cut part way through a sample.
    let truncated = &SINE_WAVE_FILE[..SINE_WAVE_FILE.len() - 101];
    let mut decoder = WavDecoder::new(std::io::Cursor::new(truncated)).unwrap();
    let mut buf = vec![0; 10000];
    let filled = decoder.next_samples(&mut buf);
    assert!(matches!(filled.stop, Some(Stop::Error(_))));
    assert_eq!(filled.written, 4411 - 51);

    let mut full = WavDecoder::new(std::io::Cursor::new(SINE_WAVE_FILE)).unwrap();
    let mut full_buf = vec![0; 10000];
    let _ = full.next_samples(&mut full_buf);
    assert_eq!(&buf[..filled.written], &full_buf[..filled.written]);
}
