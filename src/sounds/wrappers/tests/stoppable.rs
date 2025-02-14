use super::*;
use crate::tests::ConstantValueSound;

#[test]
fn set_stopped() {
    let mut first = ConstantValueSound::new(1000).stoppable();
    // starts unpaused
    assert_eq!(
        first.next_sample().unwrap(),
        crate::NextSample::Sample(1000)
    );
    first.set_stopped();
    assert_eq!(first.next_sample().unwrap(), crate::NextSample::Finished);
    assert_eq!(first.next_sample().unwrap(), crate::NextSample::Finished);
}
