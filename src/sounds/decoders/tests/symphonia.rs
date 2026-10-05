use super::*;
use crate::tests::BySample as _;
use crate::Stop;

const SINE_WAVE_FILE: &[u8] = include_bytes!("audiocheck.net_sin_1000Hz_0dBFS_0.1s.mp3");

#[test]
fn samples_of_test_file() -> std::io::Result<()> {
    let mut decoder = SymphoniaDecoder::new(Box::new(std::io::Cursor::new(SINE_WAVE_FILE)), None)
        .unwrap()
        .by_sample();
    assert_eq!(decoder.sample_rate(), 44100);
    assert_eq!(decoder.channel_count(), 1);
    for _i in 0..1 {
        let s = decoder.next_sample().unwrap();
        assert!(s.abs() < 700)
    }
    assert_eq!(decoder.next_sample().unwrap(), 4235); // 2
    assert_eq!(decoder.next_sample().unwrap(), 8784); // 3
    assert_eq!(decoder.next_sample().unwrap(), 12773); // 4
    assert_eq!(decoder.next_sample().unwrap(), 16552); // 5
    assert_eq!(decoder.next_sample().unwrap(), 20398); // 6
    assert_eq!(decoder.next_sample().unwrap(), 23584); // 7
    assert_eq!(decoder.next_sample().unwrap(), 25960); // 8
    assert_eq!(decoder.next_sample().unwrap(), 28079); // 9
    assert_eq!(decoder.next_sample().unwrap(), 29853); // 10
    assert_eq!(decoder.next_sample().unwrap(), 30799); // 11
    assert_eq!(decoder.next_sample().unwrap(), 31009); // 12
    assert_eq!(decoder.next_sample().unwrap(), 30770); // 13
    for _i in 0..4398 {
        decoder.next_sample().unwrap();
    }
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    Ok(())
}

/// Packets past the frame count in the Xing/LAME header are trimmed to zero
/// frames by Symphonia. Concatenating the file makes the header undercount,
/// producing consecutive empty packets (issue #10).
#[test]
fn consecutive_empty_packets() {
    let data = [SINE_WAVE_FILE, SINE_WAVE_FILE].concat();
    let decoder = SymphoniaDecoder::new(Box::new(std::io::Cursor::new(data)), None).unwrap();
    let sound = decoder.into_memory_sound().unwrap();
    assert_eq!(sound.channel_count(), 1);
    assert_eq!(sound.sample_rate(), 44100);
}
