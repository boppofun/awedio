use super::*;
use crate::tests::BySample as _;
use crate::Stop;

const SINE_WAVE_FILE: &[u8] = include_bytes!("audiocheck.net_sin_1000Hz_0dBFS_0.1s.qoa");

#[test]
fn samples_of_test_file() -> std::io::Result<()> {
    let mut decoder = QoaDecoder::new(std::io::Cursor::new(SINE_WAVE_FILE))
        .unwrap()
        .by_sample();
    assert_eq!(decoder.sample_rate(), 44100);
    assert_eq!(decoder.channel_count(), 1);
    assert_eq!(decoder.next_sample().unwrap(), 422); // 1
    assert_eq!(decoder.next_sample().unwrap(), 4779); // 2
    assert_eq!(decoder.next_sample().unwrap(), 8886); // 3
    assert_eq!(decoder.next_sample().unwrap(), 13834); // 4
    assert_eq!(decoder.next_sample().unwrap(), 17173); // 5
    assert_eq!(decoder.next_sample().unwrap(), 21539); // 6
    assert_eq!(decoder.next_sample().unwrap(), 24403); // 7
    assert_eq!(decoder.next_sample().unwrap(), 27482); // 8
    assert_eq!(decoder.next_sample().unwrap(), 29200); // 9
    assert_eq!(decoder.next_sample().unwrap(), 31270); // 10
    assert_eq!(decoder.next_sample().unwrap(), 31976); // 11
    assert_eq!(decoder.next_sample().unwrap(), 32767); // 12
    assert_eq!(decoder.next_sample().unwrap(), 32183); // 13
    for _i in 0..4398 {
        decoder.next_sample().unwrap();
    }
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    Ok(())
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || Box::new(QoaDecoder::new(std::io::Cursor::new(SINE_WAVE_FILE)).unwrap()),
        10000,
    );
}

/// The batch path (decode_into) must match qoaudio's sample iterator.
#[test]
fn matches_qoaudio() {
    let expected: Vec<i16> = qoaudio::decode_all(std::io::Cursor::new(SINE_WAVE_FILE))
        .unwrap()
        .samples;
    let mut decoder = QoaDecoder::new(std::io::Cursor::new(SINE_WAVE_FILE))
        .unwrap()
        .by_sample();
    let mut buf = vec![0; expected.len() + 100];
    let filled = decoder.next_samples(&mut buf);
    assert!(matches!(filled.stop, Some(crate::Stop::Finished)));
    assert_eq!(&buf[..filled.written], &expected[..]);
}
