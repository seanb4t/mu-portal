//! Option negotiation of the engine (RFC 1143).

use telnet_engine::{DO, DONT, Engine, Event, IAC, Policy, Side, WILL, WONT, opt};

fn engine() -> Engine {
    Engine::new(
        Policy::none()
            .accept(Side::Remote, &[opt::GMCP])
            .accept(Side::Local, &[opt::TTYPE]),
    )
}

fn events(engine: &mut Engine) -> Vec<Event> {
    std::iter::from_fn(|| engine.poll_event()).collect()
}

#[test]
fn accepts_an_offered_option_once() {
    let mut e = engine();
    e.receive(&[IAC, WILL, opt::GMCP]);
    assert_eq!(e.take_output(), [IAC, DO, opt::GMCP]);
    assert_eq!(events(&mut e), [Event::Enabled(Side::Remote, opt::GMCP)]);

    e.receive(&[IAC, WILL, opt::GMCP]);
    assert!(e.take_output().is_empty());
    assert!(events(&mut e).is_empty());
}

#[test]
fn refuses_options_outside_the_policy() {
    let mut e = engine();
    e.receive(&[IAC, WILL, opt::MXP, IAC, DO, opt::NAWS]);
    assert_eq!(e.take_output(), [IAC, DONT, opt::MXP, IAC, WONT, opt::NAWS]);
    assert!(events(&mut e).is_empty());
}

#[test]
fn agrees_to_perform_a_local_option() {
    let mut e = engine();
    e.receive(&[IAC, DO, opt::TTYPE]);
    assert_eq!(e.take_output(), [IAC, WILL, opt::TTYPE]);
    assert!(e.is_enabled(Side::Local, opt::TTYPE));
}

#[test]
fn an_answer_to_a_request_is_not_answered_again() {
    let mut e = engine();
    e.request(Side::Remote, opt::GMCP, true);
    assert_eq!(e.take_output(), [IAC, DO, opt::GMCP]);
    e.receive(&[IAC, WILL, opt::GMCP]);
    assert!(e.take_output().is_empty());
    assert_eq!(events(&mut e), [Event::Enabled(Side::Remote, opt::GMCP)]);
}

#[test]
fn a_refused_request_leaves_the_option_off() {
    let mut e = engine();
    e.request(Side::Remote, opt::GMCP, true);
    e.take_output();
    e.receive(&[IAC, WONT, opt::GMCP]);
    assert!(e.take_output().is_empty());
    assert!(events(&mut e).is_empty());
    assert!(!e.is_enabled(Side::Remote, opt::GMCP));
}

#[test]
fn a_queued_change_of_mind_is_sent_after_the_answer() {
    let mut e = engine();
    e.request(Side::Remote, opt::GMCP, true);
    e.request(Side::Remote, opt::GMCP, false);
    assert_eq!(e.take_output(), [IAC, DO, opt::GMCP]);
    e.receive(&[IAC, WILL, opt::GMCP]);
    assert_eq!(e.take_output(), [IAC, DONT, opt::GMCP]);
    e.receive(&[IAC, WONT, opt::GMCP]);
    assert!(e.take_output().is_empty());
    assert_eq!(
        events(&mut e),
        [
            Event::Enabled(Side::Remote, opt::GMCP),
            Event::Disabled(Side::Remote, opt::GMCP),
        ]
    );
}

#[test]
fn the_server_can_turn_an_option_off() {
    let mut e = engine();
    e.receive(&[IAC, WILL, opt::GMCP]);
    e.take_output();
    e.receive(&[IAC, WONT, opt::GMCP]);
    assert_eq!(e.take_output(), [IAC, DONT, opt::GMCP]);
    assert_eq!(
        events(&mut e),
        [
            Event::Enabled(Side::Remote, opt::GMCP),
            Event::Disabled(Side::Remote, opt::GMCP),
        ]
    );
}
