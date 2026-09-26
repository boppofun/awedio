#[cfg(test)]
mod tests {
    use crate::sounds::wrappers::Wrapper as _;
    use crate::tests::BySample as _;
    use crate::tests::{ConstantValueSound, Sawtooth};
    use crate::{Sound, Stop};

    #[test]
    fn test_constant_value_sound_basic() {
        let mut sound = ConstantValueSound::new(42).by_sample();
        assert_eq!(sound.channel_count(), 1);
        assert_eq!(sound.sample_rate(), 44100);

        // First sample should be the constant value
        assert_eq!(sound.next_sample().unwrap(), 42);
    }

    #[test]
    fn test_constant_value_sound_metadata_changes() {
        let mut sound = ConstantValueSound::new(42).by_sample();

        // Change sample rate
        sound.inner_mut().set_sample_rate(48000);
        assert_eq!(sound.sample_rate(), 48000);
        assert!(matches!(sound.next_sample(), Err(Stop::MetadataChanged)));
        assert_eq!(sound.next_sample().unwrap(), 42);

        // Change channel count
        sound.inner_mut().set_channel_count(2);
        assert_eq!(sound.channel_count(), 2);
        assert!(matches!(sound.next_frame(), Err(Stop::MetadataChanged)));
        assert_eq!(sound.next_frame().unwrap(), vec![42, 42]);

        // Multiple changes before sampling
        sound.inner_mut().set_sample_rate(96000);
        sound.inner_mut().set_channel_count(4);
        assert!(matches!(sound.next_frame(), Err(Stop::MetadataChanged)));
        assert_eq!(sound.sample_rate(), 96000);
        assert_eq!(sound.channel_count(), 4);
        assert_eq!(sound.next_frame().unwrap(), vec![42; 4]);
    }

    #[test]
    fn test_sawtooth_basic() {
        let mut sound = Sawtooth::new(1, 44100).by_sample();

        // Mono sawtooth should increment each sample
        assert_eq!(sound.next_sample().unwrap(), 0);
        assert_eq!(sound.next_sample().unwrap(), 1);
        assert_eq!(sound.next_sample().unwrap(), 2);
    }

    #[test]
    fn test_sawtooth_stereo() {
        let mut sound = Sawtooth::new(2, 44100).by_sample();

        // Stereo sawtooth should increment every other sample
        assert_eq!(sound.next_frame().unwrap(), vec![0, 0]);
        assert_eq!(sound.next_frame().unwrap(), vec![1, 1]);
    }

    #[test]
    fn test_sawtooth_wrap_around() {
        let mut sound = Sawtooth::new(1, 44100).by_sample();
        sound.inner_mut().value = i16::MAX - 1;

        assert_eq!(sound.next_sample().unwrap(), i16::MAX - 1);
        assert_eq!(sound.next_sample().unwrap(), i16::MAX);
        assert_eq!(sound.next_sample().unwrap(), i16::MIN);
    }

    #[test]
    fn test_sawtooth_sample_rate() {
        let sound = Sawtooth::new(1, 48000);
        assert_eq!(sound.sample_rate(), 48000);
    }

    #[test]
    fn test_skip() {
        let mut sound = Sawtooth::new(2, 1000).by_sample();
        assert!(sound.skip(std::time::Duration::from_millis(1500)).unwrap());
        assert_eq!(sound.next_frame().unwrap(), vec![1500, 1500]);
    }

    #[test]
    fn test_skip_stops_at_finished() {
        let mut sound = Sawtooth::new(1, 1000)
            .finish_after(std::time::Duration::from_millis(10))
            .by_sample();
        assert!(!sound.skip(std::time::Duration::from_millis(20)).unwrap());
        assert!(matches!(sound.next_sample(), Err(Stop::Finished)));
    }

    #[test]
    fn test_skip_many_channels() {
        let mut sound = Sawtooth::new(300, 1000).by_sample();
        assert!(sound.skip(std::time::Duration::from_millis(5)).unwrap());
        assert_eq!(sound.next_frame().unwrap(), vec![5; 300]);
    }
}
