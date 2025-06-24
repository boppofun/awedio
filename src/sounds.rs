//! Items that return or implement [Sound][crate::Sound].
pub mod decoders;
pub mod wrappers;

mod empty;
mod memory_sound;
mod open_bytes;
mod open_file;
mod silence;
mod sine_wave;
mod sound_list;
mod sound_mixer;
mod sounds_from_fn;

pub use empty::Empty;
pub use memory_sound::MemorySound;
pub use memory_sound::UnsupportedMetadataChangeError;
pub use open_bytes::open_bytes;
pub use open_bytes::open_reader;
pub use open_file::open_file;
pub use open_file::open_file_with_buffer_capacity;
pub use silence::Silence;
pub use sine_wave::SineWave;
pub use sound_list::SoundList;
pub use sound_mixer::SoundMixer;
pub use sounds_from_fn::SoundsFromFn;
