use crate::{Filled, Sound, Stop};
use std::io::Read;

// Enough for a single frame (maybe not for free format)
// TODO we might want to make this configurable to allow for seeking.
const INPUT_BUFFER_SIZE: usize = 2048;

/// Decoder for the MP3 format.
pub struct Mp3Decoder<R>
where
    R: Read + Send,
{
    reader: R,
    // RawDecoder is large (> 5kB) so put it on the heap
    raw_decoder: Box<rmp3::RawDecoder>,
    sample_rate: u32,
    channel_count: u16,
    input_buffer: Box<[u8; INPUT_BUFFER_SIZE]>,
    /// How many bytes of input_buffer is actually data vs just capacity
    input_buffer_data_len: usize,
    output_buffer: Box<[i16; rmp3::MAX_SAMPLES_PER_FRAME]>,
    /// How many samples are actually populated in output_buffer vs just
    /// capacity
    output_buffer_data_len: usize,
    output_buffer_next_out_idx: usize,
    metadata_changed: bool,
}

impl<R> Mp3Decoder<R>
where
    R: Read + Send,
{
    /// Attempts to decode the data as MP3.
    pub fn new(data: R) -> Mp3Decoder<R> {
        let mut decoder = Mp3Decoder {
            // TODO can we initialize this directly on the heap?
            raw_decoder: Box::new(rmp3::RawDecoder::new()),
            reader: data,
            sample_rate: 1000,
            channel_count: 1,
            input_buffer: vec![0_u8; INPUT_BUFFER_SIZE].try_into().unwrap(),
            input_buffer_data_len: 0,
            output_buffer: vec![0_i16; rmp3::MAX_SAMPLES_PER_FRAME].try_into().unwrap(),
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

impl<R> Sound for Mp3Decoder<R>
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

impl<R> Mp3Decoder<R>
where
    R: Read + Send,
{
    fn load_next_frame(&mut self) -> std::io::Result<bool> {
        loop {
            self.fill_input_buffer()?;

            let decoded = self.raw_decoder.next(
                &self.input_buffer[0..self.input_buffer_data_len],
                &mut self.output_buffer,
            );

            let Some((frame, input_bytes_to_skip)) = decoded else {
                return Ok(false);
            };

            let got_samples = match frame {
                rmp3::Frame::Audio(audio) => {
                    self.output_buffer_data_len = audio.samples().len();
                    self.output_buffer_next_out_idx = 0;
                    if self.sample_rate != audio.sample_rate() {
                        self.metadata_changed = true;
                        self.sample_rate = audio.sample_rate();
                    }
                    if self.channel_count != audio.channels() {
                        self.metadata_changed = true;
                        self.channel_count = audio.channels();
                    }
                    assert!(self.output_buffer_data_len > 0);
                    true
                }
                rmp3::Frame::Other(_) => false,
            };

            self.input_buffer
                .copy_within(input_bytes_to_skip..self.input_buffer_data_len, 0);
            self.input_buffer_data_len -= input_bytes_to_skip;

            if got_samples {
                return Ok(true);
            }
            // otherwise loop around and try again
        }
    }

    fn fill_input_buffer(&mut self) -> std::io::Result<()> {
        if self.input_buffer_data_len == self.input_buffer.len() {
            return Ok(());
        }
        let read_to: &mut [u8] = &mut self.input_buffer[self.input_buffer_data_len..];
        assert!(!read_to.is_empty());

        let num_read = self.reader.read(read_to)?;
        self.input_buffer_data_len += num_read;
        Ok(())
    }
}

#[cfg(test)]
#[path = "./tests/mp3.rs"]
mod tests;
