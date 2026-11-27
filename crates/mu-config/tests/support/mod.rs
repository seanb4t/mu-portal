//! Helpers that the tests of `layout` and `stream` share.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use proptest::prelude::*;

/// The text of a pattern: a glob of `a` to `c`, `*`, `?` and `-`, with an optional `!`.
pub fn glob() -> impl Strategy<Value = String> {
    "!?[a-c*?][a-c*?-]{0,3}"
}
