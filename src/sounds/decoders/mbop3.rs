use crate::{Filled, Sound, Stop};
use std::io::Read;

// Enough for a single frame (maybe not for free format)
const INPUT_BUFFER_SIZE: usize = 2048;

/// Decoder for the MP3 format using mbop3.
pub struct Mbop3Decoder<R>
where
    R: Read + Send,
{
    reader: R,
    decoder: Box<mbop3::Decoder>,
    sample_rate: u32,
    channel_count: u16,
    input_buffer: Box<[u8; INPUT_BUFFER_SIZE]>,
    /// How many bytes of input_buffer is actually data vs just capacity
    input_buffer_data_len: usize,
    output_buffer: Box<[i16; mbop3::MAX_SAMPLES_PER_FRAME]>,
    /// How many samples are actually populated in output_buffer vs just
    /// capacity
    output_buffer_data_len: usize,
    output_buffer_next_out_idx: usize,
    metadata_changed: bool,
}

impl<R> Mbop3Decoder<R>
where
    R: Read + Send,
{
    /// Attempts to decode the data as MP3.
    pub fn new(data: R) -> Mbop3Decoder<R> {
        let mut decoder = Mbop3Decoder {
            decoder: mbop3::Decoder::new_boxed(),
            reader: data,
            sample_rate: 1000,
            channel_count: 1,
            input_buffer: vec![0_u8; INPUT_BUFFER_SIZE].try_into().unwrap(),
            input_buffer_data_len: 0,
            output_buffer: vec![0_i16; mbop3::MAX_SAMPLES_PER_FRAME].try_into().unwrap(),
            output_buffer_data_len: 0,
            output_buffer_next_out_idx: 0,
            metadata_changed: false,
        };
        // Load the frame first so the channel_count and sample rate are set
        // appropriately
        if let Ok(true) = decoder.load_next_frame() {
            // Metadata hasn't changed since this is the first load.
            decoder.metadata_changed = false;
        };
        // If there is an error reading we will let it happen again on the first
        // next_samples call
        decoder
    }
}

impl<R> Sound for Mbop3Decoder<R>
where
    R: Read + Send,
{
    fn channel_count(&self) -> u16 {
        self.channel_count
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// An IoError from the reader (e.g. WouldBlock) is returned as a
    /// Stop::Error and decoding can continue by calling next_samples again.
    fn next_samples(&mut self, buf: &mut [i16]) -> Filled {
        let mut written = 0;
        loop {
            if self.metadata_changed {
                self.metadata_changed = false;
                return Filled::stopped(written, Stop::MetadataChanged);
            }
            let available =
                &self.output_buffer[self.output_buffer_next_out_idx..self.output_buffer_data_len];
            let to_copy = available.len().min(buf.len() - written);
            buf[written..written + to_copy].copy_from_slice(&available[..to_copy]);
            written += to_copy;
            self.output_buffer_next_out_idx += to_copy;
            if written == buf.len() {
                return Filled::all(written);
            }
            match self.load_next_frame() {
                Ok(true) => (),
                Ok(false) => return Filled::stopped(written, Stop::Finished),
                Err(e) => return Filled::stopped(written, Stop::Error(e.into())),
            }
        }
    }

    fn on_start_of_batch(&mut self) {}
}

impl<R> Mbop3Decoder<R>
where
    R: Read + Send,
{
    fn load_next_frame(&mut self) -> std::io::Result<bool> {
        loop {
            self.fill_input_buffer()?;

            let (samples, info) = self.decoder.decode_frame(
                &self.input_buffer[0..self.input_buffer_data_len],
                Some(&mut self.output_buffer),
            );
            if info.frame_bytes == 0 {
                return Ok(false);
            }
            let input_bytes_to_skip = info.frame_bytes as usize;

            let got_samples = samples > 0;
            if got_samples {
                let channels = info.channels as u16;
                let sample_rate = info.hz as u32;
                self.output_buffer_data_len = samples * channels as usize;
                self.output_buffer_next_out_idx = 0;
                if self.sample_rate != sample_rate {
                    self.metadata_changed = true;
                    self.sample_rate = sample_rate;
                }
                if self.channel_count != channels {
                    self.metadata_changed = true;
                    self.channel_count = channels;
                }
            }

            self.input_buffer
                .copy_within(input_bytes_to_skip..self.input_buffer_data_len, 0);
            self.input_buffer_data_len -= input_bytes_to_skip;

            if got_samples {
                return Ok(true);
            }
            // otherwise loop around and try again
        }
    }

    /// Fill the input buffer until it is full or the end of the reader is
    /// reached. A single read might return fewer bytes than a frame.
    fn fill_input_buffer(&mut self) -> std::io::Result<()> {
        while self.input_buffer_data_len < self.input_buffer.len() {
            let read_to: &mut [u8] = &mut self.input_buffer[self.input_buffer_data_len..];
            match self.reader.read(read_to) {
                Ok(0) => break,
                Ok(num_read) => self.input_buffer_data_len += num_read,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => (),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "./tests/mbop3.rs"]
mod tests;
