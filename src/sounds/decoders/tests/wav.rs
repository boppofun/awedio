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
