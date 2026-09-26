use crate::sounds::wrappers::Wrapper as _;
use crate::tests::BySample as _;
use crate::Sound;

use super::*;

#[test]
fn high_freq_wav() {
    let mut wav = SineWave::with_sample_rate(12000.0, 48000).by_sample();
    assert_eq!(wav.sample_rate(), 48000);
    assert_eq!(wav.channel_count(), 1);
    assert_eq!(wav.next_sample().unwrap(), i16::MAX);
    assert_eq!(wav.next_sample().unwrap(), 0);
    assert_eq!(wav.next_sample().unwrap(), i16::MIN + 1);
    assert_eq!(wav.next_sample().unwrap(), 0);
    assert_eq!(wav.next_sample().unwrap(), i16::MAX);
    assert_eq!(wav.next_sample().unwrap(), 0);
}

#[test]
fn one_khz_wav() {
    let mut wav = SineWave::with_sample_rate(1000.0, 44100).by_sample();
    assert_eq!(wav.sample_rate(), 44100);
    assert_eq!(wav.channel_count(), 1);
    assert_eq!(wav.next_sample().unwrap(), 4652); // 1
    assert_eq!(wav.next_sample().unwrap(), 9211); // 2
    assert_eq!(wav.next_sample().unwrap(), 13582); // 3
    assert_eq!(wav.next_sample().unwrap(), 17679); // 4
    assert_eq!(wav.next_sample().unwrap(), 21417); // 5
    assert_eq!(wav.next_sample().unwrap(), 24721); // 6
    assert_eq!(wav.next_sample().unwrap(), 27525); // 7
    assert_eq!(wav.next_sample().unwrap(), 29770); // 8
    assert_eq!(wav.next_sample().unwrap(), 31412); // 9
    assert_eq!(wav.next_sample().unwrap(), 32418); // 10
    assert_eq!(wav.next_sample().unwrap(), 32766); // 11
    assert_eq!(wav.next_sample().unwrap(), 32451); // 12
}

#[test]
fn high_freq_wav_memory_sound() {
    let mut wav = SineWave::as_memory_sound(12000.0, 48000).by_sample();
    assert_eq!(wav.inner().as_ref().len(), 4);
    assert_eq!(wav.sample_rate(), 48000);
    assert_eq!(wav.channel_count(), 1);
    assert_eq!(wav.next_sample().unwrap(), i16::MAX);
    assert_eq!(wav.next_sample().unwrap(), 0);
    assert_eq!(wav.next_sample().unwrap(), i16::MIN + 1);
    assert_eq!(wav.next_sample().unwrap(), 0);
    assert_eq!(wav.next_sample().unwrap(), i16::MAX);
    assert_eq!(wav.next_sample().unwrap(), 0);
}

#[test]
fn low_freq_wav_memory_sound() {
    let mut wav = SineWave::as_memory_sound(20.0, 48000).by_sample();
    assert_eq!(wav.inner().as_ref().len(), 2400);
    assert_eq!(wav.sample_rate(), 48000);
    assert_eq!(wav.channel_count(), 1);
    assert_eq!(wav.next_sample().unwrap(), 85);
}

#[test]
fn max_as_memory_sound_size() {
    let mut max_num_samples = 0;
    for hz in 20..20_000 {
        let wav = SineWave::as_memory_sound(hz as f32, 48000);
        let num_samples = wav.as_ref().len();
        if num_samples > max_num_samples {
            max_num_samples = num_samples;
        }
    }
    assert_eq!(max_num_samples, 2400);
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(|| Box::new(SineWave::new(440.0)), 10000);
}
