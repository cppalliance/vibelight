//! Facts about each declaration: what it holds, what names it, and which
//! types share a shape.

/// A chat request.
pub enum ChatDispatch {
    /// Send a message.
    Send(String),
    /// Stop the chat.
    Stop,
}

/// A tool call request.
pub enum ToolCallDispatch {
    /// Run a tool.
    Send(String),
    /// Halt.
    Stop,
}

/// A run of the engine.
pub struct Run {
    /// The name of the run.
    pub name: String,
    /// The limit, when there is one.
    pub limit: Option<u32>,
    chat: ChatDispatch,
}

impl Run {
    /// The largest run.
    pub const MAX: u32 = 9;

    /// Starts a run.
    pub fn start(name: String) -> Run {
        Run {
            name,
            limit: None,
            chat: ChatDispatch::Stop,
        }
    }

    /// Stops it.
    pub fn stop(&mut self) {
        self.chat = ChatDispatch::Stop;
    }
}

impl Default for Run {
    fn default() -> Self {
        Run::start(String::new())
    }
}

/// A point.
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// A size.
pub struct Size {
    pub x: i32,
    pub y: i32,
}

/// Starts a tool call.
pub fn dispatch(call: ToolCallDispatch) {
    let _ = call;
}
