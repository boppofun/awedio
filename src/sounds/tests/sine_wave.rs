use crate::{NextSample, Sound};

use super::*;

#[test]
fn high_freq_wav() {
    let mut wav = SineWave::with_sample_rate(12000.0, 48000);
    assert_eq!(wav.sample_rate(), 48000);
    assert_eq!(wav.channel_count(), 1);
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(i16::MAX));
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(0));
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(i16::MIN + 1));
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(0));
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(i16::MAX));
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(0));
}

#[test]
fn one_khz_wav() {
    let mut wav = SineWave::with_sample_rate(1000.0, 44100);
    assert_eq!(wav.sample_rate(), 44100);
    assert_eq!(wav.channel_count(), 1);
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(4652)); // 1
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(9211)); // 2
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(13582)); // 3
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(17679)); // 4
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(21417)); // 5
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(24721)); // 6
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(27525)); // 7
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(29770)); // 8
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(31412)); // 9
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(32418)); // 10
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(32766)); // 11
    assert_eq!(wav.next_sample().unwrap(), NextSample::Sample(32451)); // 12
}
