//! Test-support code beside production code.

/// A production engine.
pub struct Engine {
    /// The model it talks to.
    pub model: Model,
}

/// A production model.
pub struct Model;

/// Used only by test support.
pub struct Recording;

/// A chat that plays back a script, for tests and benches.
#[cfg(any(test, feature = "test-support"))]
pub struct ScriptedChat {
    /// The model the script stands in for.
    pub model: Model,
    /// Where it records.
    pub recording: Recording,
}

/// Only for tests, so the outline leaves it out.
#[cfg(test)]
pub struct OnlyTests;

/// Production only.
#[cfg(not(test))]
pub struct Live;

#[cfg(any(test, feature = "test-support"))]
pub mod fakes {
    //! Fakes for tests and benches.

    /// A fake clock.
    pub struct FakeClock;

    /// Makes a fake clock.
    pub fn clock() -> FakeClock {
        FakeClock
    }
}

/// Records into the recording.
#[cfg(any(test, feature = "test-support"))]
pub fn record(into: &mut Recording) {
    let _ = into;
}
