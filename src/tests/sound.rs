#[cfg(test)]
mod tests {
    use crate::tests::{Sawtooth, ConstantValueSound};
    use crate::{NextSample, Sound};

    #[test]
    fn test_constant_value_sound_basic() {
        let mut sound = ConstantValueSound::new(42);
        assert_eq!(sound.channel_count(), 2);
        assert_eq!(sound.sample_rate(), 44100);
        
        // First sample should be the constant value
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(42));
    }

    #[test]
    fn test_constant_value_sound_metadata_changes() {
        let mut sound = ConstantValueSound::new(42);

        // Change sample rate
        sound.set_sample_rate(48000);
        assert_eq!(sound.sample_rate(), 48000);
        assert_eq!(sound.next_sample().unwrap(), NextSample::MetadataChanged);
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(42));

        // Change channel count
        sound.set_channel_count(1);
        assert_eq!(sound.channel_count(), 1);
        assert_eq!(sound.next_sample().unwrap(), NextSample::MetadataChanged);
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(42));

        // Multiple changes before sampling
        sound.set_sample_rate(96000);
        sound.set_channel_count(4);
        assert_eq!(sound.next_sample().unwrap(), NextSample::MetadataChanged);
        assert_eq!(sound.sample_rate(), 96000);
        assert_eq!(sound.channel_count(), 4);
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(42));
    }

    #[test]
    fn test_sawtooth_basic() {
        let mut sound = Sawtooth::new(1, 44100);
        
        // Mono sawtooth should increment each sample
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(0));
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(1));
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(2));
    }

    #[test]
    fn test_sawtooth_stereo() {
        let mut sound = Sawtooth::new(2, 44100);
        
        // Stereo sawtooth should increment every other sample
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(0)); // L
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(0)); // R
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(1)); // L
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(1)); // R
    }

    #[test]
    fn test_sawtooth_wrap_around() {
        let mut sound = Sawtooth::new(1, 44100);
        sound.value = i16::MAX - 1;
        
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(i16::MAX - 1));
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(i16::MAX));
        assert_eq!(sound.next_sample().unwrap(), NextSample::Sample(i16::MIN));
    }

    #[test]
    fn test_sawtooth_sample_rate() {
        let sound = Sawtooth::new(1, 48000);
        assert_eq!(sound.sample_rate(), 48000);
    }
}