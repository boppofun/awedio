use super::*;
use crate::tests::BySample as _;
use crate::tests::ConstantValueSound;
use crate::Stop;

#[test]
fn set_paused_and_unpause() {
    let mut first = ConstantValueSound::new(1000).pausable().by_sample();
    // starts unpaused
    assert_eq!(first.next_sample().unwrap(), 1000);
    first.set_paused(true);
    assert!(matches!(first.next_sample(), Err(Stop::Paused)));
    first.set_paused(false);
    assert_eq!(first.next_sample().unwrap(), 1000);
}
