use crate::sounds::wrappers::SetPaused;
use crate::sounds::wrappers::SetVolume;
use crate::Sound;
use std::sync::mpsc;

use super::stoppable::SetStopped;
use super::AddSound;
use super::ClearSounds;
use super::SetSpeed;
use super::Wrapper;

/// Wrap a Sound so that it can be controlled via a [Controller] even after it
/// has been added to the Manager and/or started playing.
///
/// Since new sounds can be added after playing has started, if the inner sound
/// returns Finished it will be converted to Paused by Controllable. Only after
/// the controller has dropped and all sounds have played will Finished be
/// returned. If you want the Controllable to Finish immediately when the inner
/// Sound has completed, use [finish_with_inner][Controllable].
pub struct Controllable<S: Sound> {
    inner: S,
    command_receiver: mpsc::Receiver<Command<S>>,
    finished: bool,
}

impl<S> Controllable<S>
where
    S: Sound,
{
    /// Wrap `inner` so it can be controlled.
    pub fn new(inner: S) -> (Self, Controller<S>) {
        let (command_sender, command_receiver) = mpsc::channel::<Command<S>>();
        let controllable = Controllable {
            inner,
            command_receiver,
            finished: false,
        };
        let controller = Controller { command_sender };

        (controllable, controller)
    }

    /// Instead of pausing while waiting for the Controller to drop,
    /// instead finish immediately when the inner sound has finished.
    pub fn finish_with_inner(&mut self) {
        self.finished = true;
    }
}

impl<S> Sound for Controllable<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn next_sample(&mut self) -> Result<crate::NextSample, crate::Error> {
        let next = self.inner.next_sample()?;
        match next {
            crate::NextSample::Sample(_)
            | crate::NextSample::MetadataChanged
            | crate::NextSample::Paused => Ok(next),
            // Since this is controllable we might add another sound later.
            // Ideally we would do this only if the inner sound can have sounds
            // added to it but I don't think we can branch on S: AddSound here.
            // We could add a Sound::is_addable but lets avoid that until we see
            // a reason why it is necessary.
            crate::NextSample::Finished => {
                if self.finished {
                    Ok(crate::NextSample::Finished)
                } else {
                    Ok(crate::NextSample::Paused)
                }
            }
        }
    }

    fn on_start_of_batch(&mut self) {
        loop {
            match self.command_receiver.try_recv() {
                Ok(command) => command(&mut self.inner),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.finished = true;
                    break;
                }
            }
        }
        self.inner.on_start_of_batch();
    }
}

impl<S> Wrapper for Controllable<S>
where
    S: Sound,
{
    type Inner = S;

    fn inner(&self) -> &S {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }

    fn into_inner(self) -> S {
        self.inner
    }
}

// TODO consider using Small Box for perf
type Command<S> = Box<dyn FnOnce(&mut S) + Send>;

/// The remote Controller for a Sound wrapped in a Controllable.
pub struct Controller<S: Sound> {
    command_sender: mpsc::Sender<Command<S>>,
}

impl<S> Clone for Controller<S>
where
    S: Sound,
{
    fn clone(&self) -> Self {
        Self {
            command_sender: self.command_sender.clone(),
        }
    }
}

impl<S> Controller<S>
where
    S: Sound,
{
    /// Send a custom command to the associated Controllable.
    ///
    /// A command is a closure where you are given mutable access to the inner
    /// sound of the controllable. Many but not all actions are available as
    /// trait functions on `Controller` but sending a command allows for full
    /// control.
    pub fn send_command(&mut self, command: Command<S>) {
        // Ignore the error since it only happens if the receiver
        // has been dropped which is not expected after it has been
        // sent to the manager.
        let _ = self.command_sender.send(command);
    }
}

impl<S> Controller<S>
where
    S: Sound + AddSound,
{
    /// Add `sound` to the sound container.
    pub fn add(&mut self, sound: Box<dyn Sound>) {
        self.send_command(Box::new(|s: &mut S| s.add(sound)));
    }
}

impl<S> Controller<S>
where
    S: Sound + ClearSounds,
{
    /// Clear all sounds currently playing or scheduled to play.
    pub fn clear(&mut self) {
        self.send_command(Box::new(|s: &mut S| s.clear()));
    }
}

impl<S> Controller<S>
where
    S: Sound + SetPaused,
{
    /// Pause or unpause the controllable sound.
    pub fn set_paused(&mut self, paused: bool) {
        self.send_command(Box::new(move |s: &mut S| s.set_paused(paused)));
    }
}

impl<S> Controller<S>
where
    S: Sound + SetStopped,
{
    /// Stop the controllable sound (i.e. it will return Finished).
    pub fn set_stopped(&mut self) {
        self.send_command(Box::new(move |s: &mut S| s.set_stopped()));
    }
}

impl<S> Controller<S>
where
    S: Sound + SetSpeed,
{
    /// Set the playback speed of the controllable sound.
    pub fn set_speed(&mut self, speed: f32) {
        self.send_command(Box::new(move |s: &mut S| s.set_speed(speed)));
    }
}

impl<S> Controller<S>
where
    S: Sound + SetVolume,
{
    /// Set the volume of the controllable sound.
    pub fn set_volume(&mut self, volume: f32) {
        self.send_command(Box::new(move |s: &mut S| s.set_volume(volume)));
    }
}
