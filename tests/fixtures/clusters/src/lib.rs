//! Functions grouped by the fields they reach.

/// A scheduler with two groups of state.
pub struct Scheduler {
    chains: Vec<u32>,
    tasks: Vec<u32>,
    queue: Vec<u32>,
    pending: Vec<u32>,
    issued: Vec<u32>,
}

impl Scheduler {
    /// Makes one.
    pub fn new() -> Scheduler {
        Scheduler {
            chains: Vec::new(),
            tasks: Vec::new(),
            queue: Vec::new(),
            pending: Vec::new(),
            issued: Vec::new(),
        }
    }

    /// Starts a task.
    pub fn spawn(&mut self) {
        self.tasks.push(self.chains.len() as u32);
        self.queue.push(0);
    }

    /// Admits queued tasks.
    pub fn admit(&mut self) {
        let _ = self.queue.pop();
        let _ = self.chains.len();
    }

    /// Counts the tasks.
    pub fn count_tasks(&self) -> usize {
        self.tasks.len()
    }

    /// Issues an effect.
    pub fn issue(&mut self) {
        self.pending.push(1);
        self.issued.push(self.chains.len() as u32);
    }

    /// Applies an answer.
    pub fn apply(&mut self) {
        let _ = self.pending.pop();
        let _ = self.chains.first();
    }

    /// Counts the chains.
    pub fn len(&self) -> usize {
        self.chains.len()
    }
}

/// A type whose functions form one group, so it has no clusters line.
pub struct Small {
    a: u8,
    b: u8,
}

impl Small {
    /// Returns the first.
    pub fn first(&self) -> u8 {
        self.a
    }

    /// Returns both.
    pub fn both(&self) -> u8 {
        self.a + self.b
    }
}
