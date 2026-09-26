use crate::tests::BySample as _;
use crate::{
    sounds::wrappers::{SetPaused, SetSpeed},
    tests::Sawtooth,
    Sound, Stop,
};

use super::*;
use crate::sounds::wrappers::SampleBySample;
use crate::sounds::{MemorySound, SoundList};
use std::sync::Arc;

fn frames(converted: &mut SampleBySample<impl Sound>, num: usize) -> Vec<i16> {
    let mut to_return = Vec::new();
    for _ in 0..num {
        let frame = converted.next_frame().unwrap();
        assert!(frame.iter().all(|s| *s == frame[0]));
        to_return.push(frame[0]);
    }
    to_return
}

#[test]
fn test_no_conversion() {
    let sound = Sawtooth::new(2, 1000).pausable();
    let mut converted = SampleRateConverter::new(sound, 1000).by_sample();
    assert_eq!(converted.channel_count(), 2);
    assert_eq!(converted.sample_rate(), 1000);
    assert_eq!(frames(&mut converted, 3), vec![0, 1, 2]);
    converted.inner_mut().inner_mut().set_paused(true);
    assert!(matches!(converted.next_frame(), Err(Stop::Paused)));
    assert!(matches!(converted.next_frame(), Err(Stop::Paused)));
    converted.inner_mut().inner_mut().set_paused(false);
    assert_eq!(frames(&mut converted, 1), vec![3]);
}

#[test]
fn test_four_times() {
    let sound = Sawtooth::new(2, 1000).pausable();
    let mut converted = SampleRateConverter::new(sound, 250).by_sample();
    assert_eq!(converted.channel_count(), 2);
    assert_eq!(converted.sample_rate(), 250);
    assert_eq!(frames(&mut converted, 3), vec![0, 4, 8]);
    converted.inner_mut().inner_mut().set_paused(true);
    // Samples already read from the inner sound are output before the pause
    // is seen.
    let mut num_before_pause = 0;
    loop {
        match converted.next_frame() {
            Ok(_) => num_before_pause += 1,
            Err(Stop::Paused) => break,
            Err(e) => panic!("unexpected {e:?}"),
        }
    }
    assert!(num_before_pause < 100);
    assert!(matches!(converted.next_frame(), Err(Stop::Paused)));
    converted.inner_mut().inner_mut().set_paused(false);
    // Pausing does not lose any samples.
    let next = frames(&mut converted, 2);
    let expected = 12 + num_before_pause * 4;
    assert_eq!(next, vec![expected, expected + 4]);
}

#[test]
fn test_div_4() {
    let sound = Sawtooth::new(2, 1000).pausable();
    let mut converted = SampleRateConverter::new(sound, 4000).by_sample();
    assert_eq!(converted.channel_count(), 2);
    assert_eq!(converted.sample_rate(), 4000);
    // Linear interpolation between input frames: 0, 0.25, 0.5, 0.75, 1...
    // rounded to the nearest integer.
    assert_eq!(frames(&mut converted, 9), vec![0, 0, 1, 1, 1, 1, 2, 2, 2]);
}

#[test]
fn test_interpolation() {
    let samples: Vec<i16> = vec![0, 1000, 2000, 3000];
    let sound = MemorySound::from_samples(Arc::new(samples), 1, 1000);
    let mut converted = SampleRateConverter::new(sound, 4000).by_sample();
    let mut buf = [0; 20];
    let filled = converted.next_samples(&mut buf);
    // The last input frame has no next frame to interpolate with so it is held
    // for the output frames that belong to it.
    assert_eq!(filled.written, 16);
    assert!(matches!(filled.stop, Some(Stop::Finished)));
    assert_eq!(
        &buf[..16],
        &[
            0, 250, 500, 750, 1000, 1250, 1500, 1750, 2000, 2250, 2500, 2750, 3000, 3000, 3000,
            3000
        ]
    );
}

#[test]
fn catch_metadata_changed_when_passing_through() {
    let sound = Sawtooth::new(1, 1000).with_adjustable_speed();
    let mut converted = SampleRateConverter::new(sound, 1000).by_sample();
    assert_eq!(converted.channel_count(), 1);
    assert_eq!(converted.sample_rate(), 1000);
    assert_eq!(converted.next_sample().unwrap(), 0);
    assert_eq!(converted.next_sample().unwrap(), 1);
    converted.inner_mut().inner_mut().set_speed(2.0);
    assert!(matches!(
        converted.next_sample(),
        Err(Stop::MetadataChanged)
    ));
    assert_eq!(converted.sample_rate(), 1000);
    assert_eq!(converted.inner_mut().inner_mut().sample_rate(), 2000);
    assert_eq!(converted.next_sample().unwrap(), 2);
    assert_eq!(converted.next_sample().unwrap(), 4);
    assert_eq!(converted.next_sample().unwrap(), 6);
    assert_eq!(converted.channel_count(), 1);
    converted.inner_mut().inner_mut().set_speed(1.0);
    // Samples already read from inner are used before the change is seen.
    let mut last = 6;
    loop {
        match converted.next_sample() {
            Ok(s) => {
                assert_eq!(s, last + 2);
                last = s;
            }
            Err(Stop::MetadataChanged) => break,
            Err(other) => panic!("unexpected {other:?}"),
        }
    }
    // Back to passing through
    let s = converted.next_sample().unwrap();
    assert_eq!(converted.next_sample().unwrap(), s + 1);
}

#[test]
fn fill_size_independent() {
    for (from, to) in [(44100, 48000), (48000, 44100), (1000, 250), (8000, 48000)] {
        crate::tests::assert_fill_size_independent(
            || Box::new(SampleRateConverter::new(Sawtooth::new(2, from), to)),
            5000,
        );
    }
    crate::tests::assert_fill_size_independent(
        || {
            let samples: Vec<i16> = (0..3001).collect();
            let sound =
                crate::sounds::MemorySound::from_samples(std::sync::Arc::new(samples), 1, 44100);
            Box::new(SampleRateConverter::new(sound, 48000))
        },
        5000,
    );
}

/// Pull everything from `sound` in batches of 128 frames. Continues after
/// MetadataChanged. Returns the samples, and each stop with the number of
/// samples output before it and the channel count after it.
fn collect_all(sound: &mut impl Sound) -> (Vec<i16>, Vec<(String, usize, u16)>) {
    let mut out = vec![];
    let mut stops = vec![];
    loop {
        let mut buf = vec![0; 128 * sound.channel_count() as usize];
        let filled = sound.next_samples(&mut buf);
        out.extend_from_slice(&buf[..filled.written]);
        let Some(stop) = filled.stop else {
            continue;
        };
        let done = !matches!(stop, Stop::MetadataChanged);
        stops.push((format!("{stop:?}"), out.len(), sound.channel_count()));
        if done {
            return (out, stops);
        }
    }
}

fn memory_sound(samples: Vec<i16>, channel_count: u16, sample_rate: u32) -> MemorySound {
    MemorySound::from_samples(Arc::new(samples), channel_count, sample_rate)
}

/// The number of output frames for `input_frames` at `from` Hz converted to
/// `to` Hz.
fn expected_frames(input_frames: usize, from: u32, to: u32) -> usize {
    (input_frames as u64 * to as u64).div_ceil(from as u64) as usize
}

#[test]
fn output_length_does_not_drift() {
    for (from, to) in [
        (44100, 48000),
        (48000, 44100),
        (8000, 48000),
        (48000, 8000),
        (192000, 1000),
        (1000, 48000),
        // Co-prime rates so the scaled rates are large
        (44100, 48001),
    ] {
        let input_frames = from as usize * 3;
        let samples = (0..input_frames).map(|i| (i % 1000) as i16).collect();
        let mut converted = SampleRateConverter::new(memory_sound(samples, 1, from), to);
        let (out, stops) = collect_all(&mut converted);
        assert_eq!(stops.len(), 1);
        assert_eq!(
            out.len(),
            expected_frames(input_frames, from, to),
            "{from} -> {to}"
        );
    }
}

#[test]
fn tiny_sounds() {
    for input_frames in 0..4 {
        let sound = memory_sound(vec![100; input_frames], 1, 44100);
        let mut converted = SampleRateConverter::new(sound, 48000);
        let (out, _) = collect_all(&mut converted);
        assert_eq!(out.len(), expected_frames(input_frames, 44100, 48000));
        assert!(out.iter().all(|s| *s == 100));
    }
}

/// Compare with an exact linear interpolation at a non integer ratio.
#[test]
fn accurate_interpolation() {
    let (from, to) = (44100, 48000);
    // Full scale 1 kHz sine and a stereo channel with the negated signal so
    // positive and negative deltas are both checked.
    let input: Vec<f64> = (0..4410)
        .map(|i| (i as f64 * 1000.0 * std::f64::consts::TAU / from as f64).sin() * 32767.0)
        .collect();
    let samples = input
        .iter()
        .flat_map(|s| [s.round() as i16, -(s.round() as i16)])
        .collect();
    let mut converted = SampleRateConverter::new(memory_sound(samples, 2, from), to);
    let (out, _) = collect_all(&mut converted);
    let mut max_error: f64 = 0.0;
    for (k, frame) in out.as_chunks::<2>().0.iter().enumerate() {
        let position = k as f64 * from as f64 / to as f64;
        let idx = position.floor() as usize;
        let t = position - idx as f64;
        let first = input[idx].round();
        let second = input.get(idx + 1).map_or(first, |s| s.round());
        let exact = first + (second - first) * t;
        max_error = max_error.max((frame[0] as f64 - exact).abs());
        // Positive and negative interpolation are symmetric
        assert_eq!(frame[0], -frame[1], "frame {k}");
    }
    // 0.5 for rounding plus a little for the Q15 interpolation fraction
    assert!(max_error < 0.75, "max error {max_error}");
}

#[test]
fn stereo_channels_stay_separate() {
    let samples = (0..3000)
        .flat_map(|i| [i as i16, 10_000 - i as i16])
        .collect();
    let mut converted = SampleRateConverter::new(memory_sound(samples, 2, 44100), 48000);
    let (out, _) = collect_all(&mut converted);
    for frame in out.as_chunks::<2>().0.iter() {
        assert_eq!(frame[0] + frame[1], 10_000, "{frame:?}");
    }
}

#[test]
fn pause_while_converting_loses_nothing() {
    let (from, to) = (44100, 48000);
    let samples: Vec<i16> = (0..4410).map(|i| i as i16).collect();
    let reference = {
        let mut converted = SampleRateConverter::new(memory_sound(samples.clone(), 1, from), to);
        collect_all(&mut converted).0
    };

    let sound = memory_sound(samples, 1, from).pausable();
    let mut converted = SampleRateConverter::new(sound, to);
    let mut out = vec![];
    let mut buf = [0; 100];
    for call in 0.. {
        // Pause every few calls
        converted.inner_mut().set_paused(call % 5 == 3);
        let filled = converted.next_samples(&mut buf);
        out.extend_from_slice(&buf[..filled.written]);
        match filled.stop {
            None | Some(Stop::Paused) => (),
            Some(Stop::Finished) => break,
            Some(other) => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(out, reference);
}

#[test]
fn channel_count_change_while_converting() {
    let mut list = SoundList::new();
    list.add(Box::new(memory_sound((0..1000).collect(), 1, 44100)));
    list.add(Box::new(memory_sound(
        (0..1000).flat_map(|i| [i, -i]).collect(),
        2,
        44100,
    )));
    // Take the MetadataChanged of the empty list
    let _ = list.next_samples(&mut []);
    let mut converted = SampleRateConverter::new(list, 48000);
    let (out, stops) = collect_all(&mut converted);
    let mono_len = expected_frames(1000, 44100, 48000);
    assert_eq!(stops[0], ("MetadataChanged".to_owned(), mono_len, 2));
    assert_eq!(stops[1].0, "Finished");
    let stereo = &out[mono_len..];
    assert_eq!(stereo.len(), 2 * expected_frames(1000, 44100, 48000));
    assert!(stereo.as_chunks::<2>().0.iter().all(|f| f[0] == -f[1]));
}

#[test]
fn sample_rate_change_while_converting() {
    let mut list = SoundList::new();
    list.add(Box::new(memory_sound(vec![1; 44100], 1, 44100)));
    list.add(Box::new(memory_sound(vec![2; 22050], 1, 22050)));
    let _ = list.next_samples(&mut []);
    let mut converted = SampleRateConverter::new(list, 48000);
    let (out, stops) = collect_all(&mut converted);
    assert_eq!(stops[0], ("MetadataChanged".to_owned(), 48000, 1));
    assert_eq!(stops[1], ("Finished".to_owned(), 96000, 1));
    assert!(out[..48000].iter().all(|s| *s == 1));
    assert!(out[48000..].iter().all(|s| *s == 2));
}

/// Returns `remaining` samples of value 7 and then an error.
struct ErrorAfter {
    remaining: usize,
}

impl Sound for ErrorAfter {
    fn channel_count(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn next_samples(&mut self, buf: &mut [i16]) -> crate::Filled {
        let n = buf.len().min(self.remaining);
        buf[..n].fill(7);
        self.remaining -= n;
        if self.remaining == 0 {
            let error = crate::Error::IoError(std::io::Error::other("test error"));
            crate::Filled::stopped(n, Stop::Error(error))
        } else {
            crate::Filled::all(n)
        }
    }

    fn on_start_of_batch(&mut self) {}
}

#[test]
fn error_after_buffered_samples() {
    let mut converted = SampleRateConverter::new(ErrorAfter { remaining: 1000 }, 48000);
    let (out, stops) = collect_all(&mut converted);
    assert!(stops[0].0.starts_with("Error"));
    // All samples before the error are output
    assert_eq!(out.len(), expected_frames(1000, 44100, 48000));
    assert!(out.iter().all(|s| *s == 7));
}
