use super::*;
use crate::{
    sounds::SoundList,
    tests::{ConstantValueSound, DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE},
};

#[test]
fn additional_silent_sounds_do_not_affect_first() {
    let first = ConstantValueSound::new(5);
    let second = ConstantValueSound::new(0);
    let mut mixer = SoundMixer::new(DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE);
    mixer.add(Box::new(first));
    mixer.add(Box::new(second));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(5));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(5));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(5));
    let third = ConstantValueSound::new(0);
    mixer.add(Box::new(third));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(5));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(5));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(5));
}

#[test]
fn two_sounds_add_together() {
    let first = ConstantValueSound::new(5);
    let second = ConstantValueSound::new(7);
    let mut mixer = SoundMixer::new(DEFAULT_CHANNEL_COUNT, DEFAULT_SAMPLE_RATE);
    mixer.add(Box::new(first));
    mixer.add(Box::new(second));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(12));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(12));
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(12));
}

#[test]
fn empty_sound_list_not_same_sample_rate() {
    // Reporducing issue when SoundMixer matches audio but goes through SoundList
    // with different sample rate
    let mut mixer = SoundMixer::new(2, 40000);
    let (sound, mut controller) = SoundList::new().controllable();
    mixer.add(Box::new(sound));
    mixer.on_start_of_batch();
    assert_eq!(NextSample::Finished, mixer.next_sample().unwrap());
    let mut sound = ConstantValueSound::new(5);

    sound.set_channel_count(2);
    sound.set_sample_rate(40000);
    controller.add(Box::new(sound));

    assert_eq!(NextSample::Finished, mixer.next_sample().unwrap());

    mixer.on_start_of_batch();
    assert_eq!(mixer.next_sample().unwrap(), NextSample::Sample(5));
}
