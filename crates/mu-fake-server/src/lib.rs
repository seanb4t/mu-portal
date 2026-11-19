//! Fake games for `mu`. They stand in for a real game in tests and behind the
//! `--fake-world` flag.

pub mod memory;

pub use memory::{ConnectGate, FakeGame, FakeTransport, WRONG_LOGIN};
