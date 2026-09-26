use std::sync::Arc;

use super::*;
use crate::tests::BySample as _;
use crate::Stop;
use crate::{
    sounds::{MemorySound, SoundList},
    tests::{ConstantValueSound, Sawtooth, DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE},
};

#[test]
fn additional_silent_sounds_do_not_affect_first() {
    let first = ConstantValueSound::new(5);
    let second = ConstantValueSound::new(0);
    let mut mixer = SoundMixer::new(DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE).by_sample();
    mixer.add(Box::new(first));
    mixer.add(Box::new(second));
    assert_eq!(mixer.next_sample().unwrap(), 5);
    assert_eq!(mixer.next_sample().unwrap(), 5);
    assert_eq!(mixer.next_sample().unwrap(), 5);
    let third = ConstantValueSound::new(0);
    mixer.add(Box::new(third));
    assert_eq!(mixer.next_sample().unwrap(), 5);
    assert_eq!(mixer.next_sample().unwrap(), 5);
    assert_eq!(mixer.next_sample().unwrap(), 5);
}

#[test]
fn two_sounds_add_together() {
    let first = ConstantValueSound::new(5);
    let second = ConstantValueSound::new(7);
    let mut mixer = SoundMixer::new(DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE).by_sample();
    mixer.add(Box::new(first));
    mixer.add(Box::new(second));
    assert_eq!(mixer.next_sample().unwrap(), 12);
    assert_eq!(mixer.next_sample().unwrap(), 12);
    assert_eq!(mixer.next_sample().unwrap(), 12);
}

#[test]
fn saturates() {
    let mut mixer = SoundMixer::new(1, 1000).by_sample();
    mixer.add(Box::new(ConstantValueSound::new(i16::MAX - 10)));
    mixer.add(Box::new(ConstantValueSound::new(100)));
    assert_eq!(mixer.next_sample().unwrap(), i16::MAX);
}

#[test]
fn empty_sound_list_not_same_sample_rate() {
    // Reporducing issue when SoundMixer matches audio but goes through SoundList
    // with different sample rate
    let mut mixer = SoundMixer::new(2, 40000).by_sample();
    let (sound, mut controller) = SoundList::new().controllable();
    mixer.add(Box::new(sound));
    mixer.on_start_of_batch();
    assert!(matches!(mixer.next_frame(), Err(Stop::Paused)));
    let mut sound = ConstantValueSound::new(5);

    sound.set_channel_count(2);
    sound.set_sample_rate(40000);
    controller.add(Box::new(sound));

    assert!(matches!(mixer.next_frame(), Err(Stop::Paused)));

    mixer.on_start_of_batch();
    assert_eq!(mixer.next_frame().unwrap(), vec![5, 5]);
}

#[test]
fn sound_stopping_mid_batch_keeps_its_samples() {
    let mut mixer = SoundMixer::new(1, 1000).by_sample();
    mixer.add(Box::new(MemorySound::from_samples(
        Arc::new(vec![1, 2, 3]),
        1,
        1000,
    )));
    mixer.add(Box::new(MemorySound::from_samples(
        Arc::new(vec![10, 20, 30, 40, 50]),
        1,
        1000,
    )));
    let mut buf = [0; 8];
    let filled = mixer.next_samples(&mut buf);
    assert_eq!(filled.written, 5);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert_eq!(&buf[..5], &[11, 22, 33, 40, 50]);
}

#[test]
fn converts_channels_and_rate() {
    let mut mixer = SoundMixer::new(2, 2000).by_sample();
    mixer.add(Box::new(Sawtooth::new(1, 1000)));
    let mut buf = [0; 8];
    let filled = mixer.next_samples(&mut buf);
    assert!(filled.stop.is_none());
    // Frames at 0, 0.5, 1 and 1.5 of the input with 0.5 rounded up
    assert_eq!(buf, [0, 0, 1, 1, 1, 1, 2, 2]);
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || {
            let mut mixer = SoundMixer::new(2, 48000).by_sample();
            mixer.add(Box::new(Sawtooth::new(2, 48000)));
            mixer.add(Box::new(Sawtooth::new(1, 48000)));
            mixer.add(Box::new(Sawtooth::new(2, 44100)));
            let samples: Vec<i16> = (0..1001).collect();
            mixer.add(Box::new(MemorySound::from_samples(
                Arc::new(samples),
                1,
                48000,
            )));
            Box::new(mixer)
        },
        3000,
    );
}
