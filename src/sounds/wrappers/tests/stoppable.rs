use super::*;
use crate::tests::BySample as _;
use crate::tests::ConstantValueSound;
use crate::Stop;

#[test]
fn set_stopped() {
    let mut first = ConstantValueSound::new(1000).stoppable().by_sample();
    // starts unpaused
    assert_eq!(first.next_sample().unwrap(), 1000);
    first.set_stopped();
    assert!(matches!(first.next_sample(), Err(Stop::Finished)));
    assert!(matches!(first.next_sample(), Err(Stop::Finished)));
}
