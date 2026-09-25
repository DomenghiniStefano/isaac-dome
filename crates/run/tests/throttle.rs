//! Card #80, item 02: acting on a burst of log changes. The watcher acted on the first change
//! and dropped every one inside the next two seconds, so the last lines of a burst — a death,
//! an ending, the game closing — stayed unread, and were lost for good when the next touch was
//! a relaunch. The rule, from the card: act at once on a change, and never drop one — a change
//! inside the quiet period is acted on when the period ends.

use std::time::{Duration, Instant};

use run::Throttle;

const PERIOD: Duration = Duration::from_secs(2);

fn at(start: Instant, millis: u64) -> Instant {
    start + Duration::from_millis(millis)
}

#[test]
fn the_first_change_is_acted_on_at_once() {
    let t0 = Instant::now();
    let mut t = Throttle::new(PERIOD);
    assert!(t.event(t0));
    assert_eq!(t.wait(t0), None, "nothing is left pending");
}

#[test]
fn a_change_inside_the_period_is_kept_and_acted_on_when_it_ends() {
    let t0 = Instant::now();
    let mut t = Throttle::new(PERIOD);
    assert!(t.event(t0));
    assert!(!t.event(at(t0, 500)), "too soon to act again");
    assert_eq!(t.wait(at(t0, 500)), Some(Duration::from_millis(1500)));
    assert!(!t.tick(at(t0, 1999)), "the period has not ended");
    assert!(t.tick(at(t0, 2000)), "the kept change is acted on");
    assert_eq!(t.wait(at(t0, 2000)), None);
}

#[test]
fn a_whole_burst_inside_the_period_is_one_trailing_action() {
    let t0 = Instant::now();
    let mut t = Throttle::new(PERIOD);
    assert!(t.event(t0));
    for ms in [100, 400, 900, 1500, 1900] {
        assert!(!t.event(at(t0, ms)));
    }
    assert!(t.tick(at(t0, 2000)));
    assert!(!t.tick(at(t0, 4000)), "one burst, one trailing action");
}

#[test]
fn a_change_after_the_period_is_acted_on_at_once() {
    let t0 = Instant::now();
    let mut t = Throttle::new(PERIOD);
    assert!(t.event(t0));
    assert!(t.event(at(t0, 2500)));
}

#[test]
fn with_no_change_there_is_nothing_to_act_on() {
    let t0 = Instant::now();
    let mut t = Throttle::new(PERIOD);
    assert_eq!(t.wait(t0), None);
    assert!(!t.tick(at(t0, 10_000)));
}
