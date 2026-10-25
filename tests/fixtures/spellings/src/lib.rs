//! Spellings: items from outside the crate written in more than one way.

use std::collections::BTreeMap as Map;
use std::fmt;

use serde::Serialize;

/// A clock with an optional alarm.
#[derive(Serialize)]
pub struct Clock {
    /// When the alarm rings; see `core::option::Option`.
    pub alarm: Option<u8>,
    pub label: String,
    pub snooze: Option<u8>,
    pub step: core::time::Duration,
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// A timer that names its laps.
#[derive(serde::Serialize)]
pub struct Timer {
    pub laps: Map<u32, String>,
    pub names: std::collections::BTreeMap<u32, String>,
    pub period: ::std::time::Duration,
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.laps.len())
    }
}

impl Timer {
    pub fn next(&self, clock: &crate::Clock) -> std::option::Option<Clock> {
        let unused: core::option::Option<u8> = clock.alarm;
        unused.map(|_| None).unwrap_or(None)
    }

    pub fn wait(&self, extra: std::time::Duration) -> Vec<String> {
        let _ = extra;
        Vec::new()
    }
}

/// A wrapper whose impl names std::string::String only in its self type,
/// which does not count.
pub struct Wrapper<T> {
    pub inner: T,
}

impl Wrapper<std::string::String> {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
