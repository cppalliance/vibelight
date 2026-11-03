//! Enums whose variant names overlap.

/// Where a task is in its run.
pub enum Phase {
    Queued,
    Running,
    Done,
}

/// A task's state.
pub enum State {
    Queued,
    Running,
    Done,
    Failed,
}

/// A job's state.
pub enum Job {
    Queued,
    Running,
    Done,
    Paused,
    Resumed,
    Cancelled,
    Archived,
}

/// How a probe of a connection ended.
pub enum Probe {
    Dial,
    Handshake,
    Ready,
    Timeout,
}

/// How an attempt to connect ended.
pub enum Attempt {
    Dial,
    Handshake,
    Ready,
    Refused,
}

/// A request to a file service.
pub enum Request {
    Open,
    Fetch,
    Store,
    Close,
    Flush,
    Stat,
}

/// The answer to a request.
pub enum Answer {
    Open,
    Fetch,
    Store,
    Close,
    Opened,
    Fetched,
    Stored,
}
