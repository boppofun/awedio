use super::*;
use crate::tests::BySample as _;
use crate::Stop;

const SINE_WAVE_FILE: &[u8] = include_bytes!("audiocheck.net_sin_1000Hz_0dBFS_0.1s.mp3");

#[test]
fn samples_of_test_file1() -> std::io::Result<()> {
    let mut decoder = Mbop3Decoder::new(std::io::Cursor::new(SINE_WAVE_FILE)).by_sample();
    assert_eq!(decoder.sample_rate(), 44100);
    assert_eq!(decoder.channel_count(), 1);
    for _i in 0..2258 {
        // println!("i: {_i}");
        let s = decoder.next_sample().unwrap();
        // println!("s: {s}");
        assert!(s.abs() < 700)
    }
    assert_eq!(decoder.next_sample().unwrap(), 4235); // 2
    assert_eq!(decoder.next_sample().unwrap(), 8784); // 3
    assert_eq!(decoder.next_sample().unwrap(), 12774); // 4
    assert_eq!(decoder.next_sample().unwrap(), 16553); // 5
    assert_eq!(decoder.next_sample().unwrap(), 20398); // 6
    assert_eq!(decoder.next_sample().unwrap(), 23584); // 7
    assert_eq!(decoder.next_sample().unwrap(), 25961); // 8
    assert_eq!(decoder.next_sample().unwrap(), 28080); // 9
    assert_eq!(decoder.next_sample().unwrap(), 29853); // 10
    assert_eq!(decoder.next_sample().unwrap(), 30800); // 11
    assert_eq!(decoder.next_sample().unwrap(), 31010); // 12
    assert_eq!(decoder.next_sample().unwrap(), 30771); // 13
    for _i in 0..4642 {
        // println!("i: {_i}");
        decoder.next_sample().unwrap();
    }
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    Ok(())
}

#[test]
#[cfg(not(debug_assertions))] // rmp3 has a read out of bounds. We need to switch to another decoder
fn samples_of_test_file2() -> std::io::Result<()> {
    const STEREO_FILE: &[u8] = include_bytes!("../../../../test_files/stereo-test.mp3");

    let mut decoder = Mbop3Decoder::new(std::io::Cursor::new(STEREO_FILE)).by_sample();
    assert_eq!(decoder.sample_rate(), 32000);
    assert_eq!(decoder.channel_count(), 2);
    for _i in 0..1_078_272 {
        decoder.next_sample().unwrap();
    }
    assert!(matches!(decoder.next_sample(), Err(Stop::Finished)));
    Ok(())
}

#[test]
fn fill_size_independent() {
    crate::tests::assert_fill_size_independent(
        || Box::new(Mbop3Decoder::new(std::io::Cursor::new(SINE_WAVE_FILE))),
        10000,
    );
}

/// Returns at most 100 bytes per read and every other read is Interrupted.
struct ShortReader {
    inner: std::io::Cursor<&'static [u8]>,
    interrupt: bool,
}

impl std::io::Read for ShortReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.interrupt = !self.interrupt;
        if self.interrupt {
            return Err(std::io::ErrorKind::Interrupted.into());
        }
        let len = buf.len().min(100);
        self.inner.read(&mut buf[..len])
    }
}

#[test]
fn short_reads() {
    let (expected, expected_stops) = crate::tests::collect(
        &mut Mbop3Decoder::new(std::io::Cursor::new(SINE_WAVE_FILE)),
        10000,
        1000,
    );
    let mut decoder = Mbop3Decoder::new(ShortReader {
        inner: std::io::Cursor::new(SINE_WAVE_FILE),
        interrupt: false,
    });
    let (samples, stops) = crate::tests::collect(&mut decoder, 10000, 1000);
    assert_eq!(stops, expected_stops);
    assert_eq!(samples, expected);
}
