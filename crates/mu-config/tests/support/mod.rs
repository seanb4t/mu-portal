//! Helpers that the tests of `layout` and `stream` share.

#![allow(dead_code, reason = "each test binary uses a part of this module")]

use mu_config::layout::BlockId;
use proptest::prelude::*;

/// The block id `text`.
pub fn id_of(text: &str) -> BlockId {
    BlockId::new(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// The text of a pattern: a glob of `a` to `c`, `*` and `?`, then also `0` to `9` and `-`,
/// with an optional `!`.
pub fn glob() -> impl Strategy<Value = String> {
    "!?[a-c*?][a-c0-9*?-]{0,3}"
}
