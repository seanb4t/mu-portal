//! The reply of one send: a refusal when the call fails or gives no answer in time.

use std::time::Duration;

use mu_tui::{CALL_LIMIT, Reply, reply_of, within};

const LIMIT: Duration = Duration::from_secs(3);

#[tokio::test(start_paused = true)]
async fn send_with_no_answer_is_refused() {
    let call = std::future::pending::<Result<(), String>>();
    let reply = reply_of(call, "alpha/aria".to_owned(), "one".to_owned(), CALL_LIMIT).await;
    let Reply::Refused { reason, text, .. } = reply else {
        panic!("not refused: {reply:?}");
    };
    assert_eq!(text, "one");
    assert!(reason.contains("no answer in 30 s"), "{reason}");
}

#[tokio::test]
async fn failed_send_is_refused_with_its_reason() {
    let call = async { Err::<(), _>("not connected".to_owned()) };
    let reply = reply_of(call, "alpha/aria".to_owned(), "one".to_owned(), LIMIT).await;
    assert_eq!(
        reply,
        Reply::Refused {
            world: "alpha/aria".to_owned(),
            text: "one".to_owned(),
            reason: "not connected".to_owned(),
        }
    );
}

#[tokio::test]
async fn answered_send_is_sent() {
    let reply = reply_of(
        async { Ok(()) },
        "alpha/aria".to_owned(),
        "one".to_owned(),
        LIMIT,
    )
    .await;
    assert_eq!(
        reply,
        Reply::Sent {
            world: "alpha/aria".to_owned(),
            text: "one".to_owned(),
        }
    );
}

/// A call that answers `Ok` after `delay` on the clock of the runtime.
async fn answer_after(delay: Duration) -> Result<(), String> {
    tokio::time::sleep(delay).await;
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn answer_just_inside_the_limit_is_sent() {
    let call = answer_after(CALL_LIMIT.saturating_sub(Duration::from_millis(1)));
    let reply = reply_of(call, "alpha/aria".to_owned(), "one".to_owned(), CALL_LIMIT).await;
    assert!(matches!(reply, Reply::Sent { .. }), "{reply:?}");
}

#[tokio::test(start_paused = true)]
async fn answer_just_past_the_limit_is_refused() {
    let call = answer_after(CALL_LIMIT + Duration::from_millis(1));
    let reply = reply_of(call, "alpha/aria".to_owned(), "one".to_owned(), CALL_LIMIT).await;
    assert!(matches!(reply, Reply::Refused { .. }), "{reply:?}");
}

#[tokio::test(start_paused = true)]
async fn list_with_no_answer_fails() {
    let call = std::future::pending::<Result<(), String>>();
    let listed = within(call, CALL_LIMIT, "").await;
    assert_eq!(listed, Err("the server gave no answer in 30 s".to_owned()));
}
