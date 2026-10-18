//! Manager is how sounds are played on a backend.
mod backend_source;
mod renderer;

use crate::sounds::wrappers::Controllable;
use crate::sounds::wrappers::Controller;
use crate::sounds::SoundMixer;
use crate::Sound;
pub use backend_source::BackendSource;
pub use renderer::Renderer;
use renderer::RootMixer;

/// A Manager can play sounds by rendering sounds on a [`Renderer`] for a
/// backend.
#[derive(Clone)]
pub struct Manager {
    controller: Controller<RootMixer>,
}

// These are undocumented, should not be relied on and subject to change.
// Backend implementations should set their values at startup.
const DEFAULT_CHANNEL_COUNT: u16 = 1;
const DEFAULT_SAMPLE_RATE: u32 = 1000; // Purposely low value to discourage use

impl Manager {
    /// Create a new Manager and the renderer its samples will render to.
    ///
    /// Normally you do not need to call this function directly but you instead
    /// call `.start(...)` on a backend which will call this function.
    pub fn new() -> (Self, Renderer) {
        let (root, controller) = Controllable::new(RootMixer::new(SoundMixer::new(
            DEFAULT_CHANNEL_COUNT,
            DEFAULT_SAMPLE_RATE,
        )));
        let renderer = Renderer::new(root);
        let manager = Manager { controller };
        (manager, renderer)
    }

    /// Add a new Sound to be played in parallel to any existing sounds.
    ///
    /// If you want to play Sounds sequentially use a
    /// [SoundList][crate::sounds::SoundList].
    ///
    /// See the modifier functions on [Sound] to control sounds before and/or
    /// after playing.
    pub fn play(&mut self, sound: Box<dyn Sound>) {
        self.controller.add(sound);
    }

    /// Stop playing and remove all audio sounds. New sounds can still be added.
    pub fn clear(&mut self) {
        self.controller.clear();
    }

    /// Call `f` on the thread rendering the sounds at the start of every
    /// batch, before any sound renders. Replaces a previously set `f`.
    ///
    /// For example to mark the rendering thread with a thread local, which
    /// keeps working if the backend renders on a new thread.
    pub fn set_on_start_of_batch(&mut self, f: Box<dyn FnMut() + Send>) {
        self.controller
            .send_command(Box::new(move |root: &mut RootMixer| {
                root.on_start_of_batch = Some(f)
            }));
    }
}

impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager").finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::Manager;
    use crate::Sound;

    #[test]
    fn on_start_of_batch_runs_every_batch() {
        let (mut manager, mut renderer) = Manager::new();
        let count = Arc::new(AtomicUsize::new(0));
        let counter = count.clone();
        manager.set_on_start_of_batch(Box::new(move || {
            counter.fetch_add(1, Ordering::Relaxed);
        }));
        assert_eq!(count.load(Ordering::Relaxed), 0);
        renderer.on_start_of_batch();
        assert_eq!(count.load(Ordering::Relaxed), 1);
        renderer.on_start_of_batch();
        assert_eq!(count.load(Ordering::Relaxed), 2);
    }
}
