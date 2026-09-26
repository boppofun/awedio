use super::Wrapper;
use crate::Sound;
use std::sync::mpsc;

/// Notify via a [std::sync::mpsc::Receiver] when the contained Sound has
/// Finished. A single message is sent when the sound has completed.
///
/// If the Sound is dropped before it returned Finished then the receiver
/// will return an error. The contained Sound pausing or yielding an error
/// does not count as completion.
///
/// See also [crate::Sound::with_async_completion_notifier]
/// See also [super::AsyncCompletionNotifier]
pub struct CompletionNotifier<S: Sound> {
    inner: S,
    sender: Option<mpsc::SyncSender<()>>,
}

impl<S> CompletionNotifier<S>
where
    S: Sound,
{
    /// Wrap `inner` so a receiver can be notified when `inner` has `Finished`.
    pub fn new(inner: S) -> (Self, mpsc::Receiver<()>) {
        let (sender, receiver) = mpsc::sync_channel(1);
        let controllable = CompletionNotifier {
            inner,
            sender: Some(sender),
        };

        (controllable, receiver)
    }
}

impl<S> Sound for CompletionNotifier<S>
where
    S: Sound,
{
    fn channel_count(&self) -> u16 {
        self.inner.channel_count()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    #[inline]
    fn next_samples(&mut self, buf: &mut [i16]) -> crate::Filled {
        let filled = self.inner.next_samples(buf);
        if let Some(crate::Stop::Finished) = filled.stop {
            if let Some(sender) = self.sender.take() {
                // If the consumer dropped their receiver because they don't need it anymore its
                // not an error.
                let _res = sender.send(());
            }
        }
        filled
    }

    fn on_start_of_batch(&mut self) {
        self.inner.on_start_of_batch();
    }
}

impl<S> Wrapper for CompletionNotifier<S>
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
