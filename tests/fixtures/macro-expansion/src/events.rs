//! Events whose variants all carry the same stamp field first.

/// Declares the event enum and its accessor: every variant carries
/// `execution` first, then its own fields.
macro_rules! events {
    ($( $(#[$meta:meta])* $variant:ident { $($field:ident : $ty:ty),* $(,)? } ),* $(,)?) => {
        /// One thing that happened during a run.
        #[derive(Debug, Clone)]
        pub enum Event {
            $(
                $(#[$meta])*
                $variant {
                    /// The run that produced the event, as the scheduler names it in every log line.
                    execution: String,
                    $($field: $ty),*
                },
            )*
        }

        impl Event {
            /// Returns the run that produced the event.
            pub fn execution(&self) -> &str {
                match self {
                    $( Event::$variant { execution, .. } )|* => execution,
                }
            }
        }
    };
}

events! {
    /// The run started.
    Started {},
    Stopped { code: i32 },
    Moved { from: u32, to: u32 },
}
