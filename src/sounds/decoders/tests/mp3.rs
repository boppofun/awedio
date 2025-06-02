use super::*;
use crate::NextSample;

const SINE_WAVE_FILE: &[u8] = include_bytes!("audiocheck.net_sin_1000Hz_0dBFS_0.1s.mp3");
const STEREO_FILE: &[u8] = include_bytes!("../../../../test_files/stereo-test.mp3");

#[test]
fn samples_of_test_file1() -> std::io::Result<()> {
    let mut decoder = Mp3Decoder::new(std::io::Cursor::new(SINE_WAVE_FILE));
    assert_eq!(decoder.sample_rate(), 44100);
    assert_eq!(decoder.channel_count(), 1);
    for _i in 0..2258 {
        // println!("i: {_i}");
        let sample = decoder.next_sample().unwrap();
        match sample {
            NextSample::Sample(s) => {
                // println!("s: {s}");
                assert!(s.abs() < 700)
            }
            NextSample::MetadataChanged => unreachable!(),
            NextSample::Paused => unreachable!(),
            NextSample::Finished => unreachable!(),
        }
    }
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(4235)); // 2
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(8784)); // 3
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(12774)); // 4
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(16553)); // 5
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(20398)); // 6
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(23584)); // 7
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(25961)); // 8
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(28080)); // 9
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(29853)); // 10
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(30800)); // 11
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(31010)); // 12
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Sample(30771)); // 13
    for _i in 0..4642 {
        // println!("i: {_i}");
        let sample = decoder.next_sample().unwrap();
        match sample {
            NextSample::Sample(_) => {}
            NextSample::MetadataChanged => unreachable!(),
            NextSample::Paused => unreachable!(),
            NextSample::Finished => unreachable!(),
        }
    }
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Finished);
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Finished);
    Ok(())
}

#[test]
fn samples_of_test_file2() -> std::io::Result<()> {
    let mut decoder = Mp3Decoder::new(std::io::Cursor::new(STEREO_FILE));
    assert_eq!(decoder.sample_rate(), 32000);
    assert_eq!(decoder.channel_count(), 2);
    for _i in 0..1_078_272 {
        let sample = decoder.next_sample().unwrap();
        match sample {
            NextSample::Sample(_s) => {}
            NextSample::MetadataChanged => unreachable!(),
            NextSample::Paused => unreachable!(),
            NextSample::Finished => unreachable!(),
        }
    }
    assert_eq!(decoder.next_sample().unwrap(), NextSample::Finished);
    Ok(())
}
