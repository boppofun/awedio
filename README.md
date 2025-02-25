# Awedio &emsp; [![Docs Passing]][docs.rs] [![Latest Version]][crates.io]

A low-overhead and adaptable audio playback library for Rust.

By default supports playing mp3, wav, flac, aac, ogg, qoa and all other formats
supported by [Symphonia](https://crates.io/crates/symphonia).

## Examples

Play a single sound file:

```rust
let (mut manager, backend) = awedio::start()?;
manager.play(awedio::sounds::open_file("test_files/stereo-test.mp3")?);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Play a sound with adjustable volume controllable after playback has started:

```rust
use awedio::Sound;
let (mut manager, backend) = awedio::start()?;
let (sound, mut controller) = awedio::sounds::SineWave::new(400.0)
    .with_adjustable_volume_of(0.25)
    .pausable()
    .controllable();
manager.play(Box::new(sound));
std::thread::sleep(std::time::Duration::from_millis(100));
controller.set_volume(0.5);
std::thread::sleep(std::time::Duration::from_millis(100));
controller.set_paused(true);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Design Goals

- Modular design. Easy to add new backends, wrappers and decoders.
- Very low overhead. For example, a Wav file with i16 samples with the same
  same sample rate as the output device will have samples sent to the backend
  unchanged.
- Only pay the performance cost of features if they are needed. For example
  pausability, volume adjustment, or controlling a sound after playback has
  started are all added to a Sound only as needed. This is done by wrapping
  types that implement `Sound` similar to `Iterator` in the standard library.
- Usable in low resource environments such as the esp32 microcontroller.
  Currently does require std.
- Samples are i16 for simplicity at least for now (if this prevents you from
  using this library please let me know your use case)

## API Overview

- [Sound] trait - Provides samples of a sound to be played. Has
  functions to modify sounds with wrappers.
- [Manager] - Play Sounds to a backend.
- [SoundList] - A sequence of Sounds to play one after the other.

## Current backends

- [cpal] - For popular environments such as Linux,
  Windows, Mac OS, Android...  Enabled by the `cpal` feature (on by default).
- [esp32][awedio_esp32] - For esp32 microcontrollers using
  esp-idf. Implemented in its [own crate][awedio_esp32].

Backends are implemented by pulling samples from a
[BackendSource][crate::manager::BackendSource] such as the [Renderer].

## Cargo Features

- `async`: Enable async features that depend on
  [tokio-sync](https://docs.rs/tokio/latest/tokio/sync/index.html)
- `cpal`: Enable the [cpal] backend.
- `symphonia-all`: Enable all formats and codecs supported by
  [Symphonia](https://crates.io/crates/symphonia)
- `symphonia-`: All feature flags of symphonia are re-exported with the `symphonia-` prefix.
- `hound-wav`: Enable wav decoding using [Hound](https://crates.io/crates/hound)
- `rmp3-mp3`: Enable mp3 decoding using [rmp3](https://crates.io/crates/rmp3)
- `qoa`: Enable qoa decoding using [qoaudio](https://crates.io/crates/qoaudio)

By default all features are enabled excluding `hound-wav` and `rmp3-mp3`
since symphonia handles those formats by default.

Depending libraries should disable default features.

## Motivation

Built for creating activities for [boppo](https://boppo.com), a
screenless tablet for kids. Purposefully kept generic to be usable in other
contexts.

## Alternatives and Inspiration

Thanks to the following audio playback libraries which inspired and were
a reference for this library:

- [Rodio](https://docs.rs/rodio/)
  - Very popular crate for audio playback with Rust.
  - Tightly coupled to cpal. Awedio allows for easy integration with cpal or
    other backends.
  - Has a [Sink](https://docs.rs/rodio/latest/rodio/#sink) which is similar to
    `SoundList::new().pausable().with_adjustable_volume().controllable()`...
  - Mixer converts everything to an f32. Awedio uses i16 consistently.
  - Sources have the concept of a frame length where metadata should not change
    but this does not work well for some wrappers such as Speed.
  - Uses the standard Iterator trait for its Source. Awedio's Sound is its own
    enum that handles Metadata Changes, Pausing and Finishing sounds.
  - Does not have a way to propagate errors through Sources. Awedio's sound
    returns a result so errors can be explicitly handled if desired.
- [Kira](https://docs.rs/kira/)
  - Has very nice API for tweening and effects.
  - [Sound](https://docs.rs/kira/latest/kira/sound/trait.Sound.html) trait
    requires a left/right track instead of supporting 1 or N tracks.
  - Samples are all f32. Thus frames are always 8 bytes. A frame for a mono
    Sound in awedio is only 2 bytes.
  - Uses symphonia for audio format parsing and has several internal buffers
    requiring more memory. Awedio optionally uses symphonia or other decoding
    crates with less buffering required.
  - All samples are resampled based on the timestamp and sample rate. Awedio
    only resamples if the source and output rates do not match.

## License

This project is licensed under either of
[Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT license](https://opensource.org/licenses/MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[Latest Version]: https://img.shields.io/crates/v/awedio.svg
[crates.io]: https://crates.io/crates/awedio
[Docs Passing]: https://img.shields.io/docsrs/awedio.svg
[docs.rs]: https://docs.rs/awedio
[cpal]: https://crates.io/crates/cpal
[awedio_esp32]: https://crates.io/crates/awedio_esp32
[Sound]: https://docs.rs/awedio/latest/awedio/trait.Sound.html
[Manager]: https://docs.rs/awedio/latest/awedio/manager/struct.Manager.html
[SoundList]: https://docs.rs/awedio/latest/awedio/sounds/struct.SoundList.html
[Renderer]: https://docs.rs/awedio/latest/awedio/manager/struct.Renderer.html
