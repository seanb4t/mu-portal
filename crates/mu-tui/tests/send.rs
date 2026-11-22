//! The reply of one send: a refusal when the call fails or gives no answer in time.

use std::time::Duration;

use mu_tui::{Reply, SEND_LIMIT, reply_of};

const LIMIT: Duration = Duration::from_secs(3);

#[tokio::test(start_paused = true)]
async fn send_with_no_answer_is_refused() {
    let call = std::future::pending::<Result<(), String>>();
    let reply = reply_of(call, "alpha/aria".to_owned(), "one".to_owned(), SEND_LIMIT).await;
    let Reply::Refused { reason, text, .. } = reply else {
        panic!("not refused: {reply:?}");
    };
    assert_eq!(text, "one");
    assert!(reason.contains("no answer in 10 s"), "{reason}");
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
