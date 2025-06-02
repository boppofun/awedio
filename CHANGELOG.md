## [0.5.0] - 2025-06-02

### Features

- Add stoppable to Sound
- Add Controllable::finish_with_inner
- Add Empty sound
- Add SoundList::insert and SoundList::len

### Bug Fixes

- Sine wav off by one and error accumulation
- [**breaking**] Rename SineWav to SineWave

### Documentation

- Fix links
- Update README examples to execute as tests
- Improve docs for adjustable volume
- Apply cargo-fmt line wrap
- Rename github org to boppofun

### Testing

- Add another mp3 file test

# Changelog

## Unreleased

Unreleased changes, if any, can be listed using `git log` or `git cliff -u`.

## [0.4.1] - 2024-07-31

### Bug Fixes

- Add device output stream sample format convert. Fix Mac OS output.

## [0.4.0] - 2024-05-29

### Features

- Add BackendSource trait for Renderer
- Update cpal and qoaudio deps


## [0.3.2] - 2024-05-22

### Features

- Implement Debug for SoundList
- Add count_samples example
- Add Sound::skip

### Bug Fixes

- Add cpal as required feature for play example
- recover from some errors in SymphoniaDecoder

### Documentation

- Improve Renderer::next_sample doc
- Add more for completion notifiers

### Performance

- Drop old sound before creating new in SoundsFromFn


## [0.3.1] - 2023-12-07

### Bug Fixes

- Errors when compiling without default features


## [0.3.0] - 2023-12-07

### Bug Fixes

- Clear paused sounds too in SoundMixer

### Documentation

- Add basic play example in examples/
- Fix ambiguous doc reference
- Update README

### Features

- Make Controller::send_command public
- Add QOA audio format decoder
- Add Sound::paused which starts paused
- Re-export tokio oneshot in async_completion_notifier mod
- Add CompletionNotifier (blocking, non-async)
- Add finish after sound wrapper
- [**breaking**] Add symphonia decoder
- [**breaking**] Change Sound::next_sample to return Result

### Performance

- Do not round in AdjustableVolume

### Refactor

- Fixes for clippy, cspell, cargo format


## [0.2.0] - 2023-05-11

- update README links
- rename UnexpectedMetadataChange to UnsupportedMetadataChangeError


## [0.1.1] - 2023-05-10

- Improved documentation


## [0.1.0] - 2023-05-09

- Initial release
