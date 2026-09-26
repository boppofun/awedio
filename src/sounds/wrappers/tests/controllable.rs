use crate::tests::BySample as _;
use crate::tests::ConstantValueSound;

use super::*;
use crate::Stop;

#[test]
fn finished_becomes_paused_until_controller_dropped() {
    let (sound, controller) = crate::sounds::Empty::new(1, 1000).controllable();
    let mut sound = sound.by_sample();
    sound.on_start_of_batch();
    assert!(matches!(sound.next_sample(), Err(Stop::Paused)));
    drop(controller);
    sound.on_start_of_batch();
    assert!(matches!(sound.next_sample(), Err(Stop::Finished)));
}

#[test]
fn commands_apply_at_start_of_batch() {
    let (sound, mut controller) = ConstantValueSound::new(100)
        .with_adjustable_volume()
        .controllable();
    let mut sound = sound.by_sample();
    assert_eq!(sound.next_sample().unwrap(), 100);
    controller.set_volume(0.5);
    assert_eq!(sound.next_sample().unwrap(), 100);
    sound.on_start_of_batch();
    assert_eq!(sound.next_sample().unwrap(), 50);
}
