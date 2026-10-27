//! Dependency order: each part follows its owner.

/// The main entry: it reaches the most declarations.
pub struct Run {
    pub config: Config,
    pub steps: Vec<Step>,
}

/// A diamond root: it reaches three declarations through two branches.
pub struct Fork {
    pub a_left: Left,
    pub b_right: Right,
}

/// A root whose public method prints before its private field, so its
/// block names Job first, yet its field type, Store, still comes first.
pub struct Host {
    store: Store,
}

impl Host {
    /// Runs a job.
    pub fn run(&self, job: Job) {
        let _ = job;
    }
}

/// Named only by Host's field.
pub struct Store;

/// Named only by Host's method.
pub struct Job;

/// A root that ties Report on reach and comes first by path.
pub struct Able {
    pub leaf: Leaf,
}

/// A root that ties Able on reach.
pub struct Report {
    pub summary: Summary,
}

/// Named by Run and by Step, so it follows Step's last part, not Run.
pub struct Config {
    pub limit: u32,
}

/// Named by Run; first names Trace, then Log, then Config.
pub struct Step {
    pub a_trace: Trace,
    pub b_log: Log,
    pub config: Config,
}

/// Named only by Step.
pub struct Log;

/// Named only by Step, before Log.
pub struct Trace;

/// Named only by Fork.
pub struct Left {
    pub base: Base,
}

/// Named only by Fork, after Left.
pub struct Right {
    pub base: Base,
}

/// Named by Left and by Right, so it follows Right, the last of them.
pub struct Base;

/// Named only by Able.
pub struct Leaf;

/// Named only by Report.
pub struct Summary;

/// Names nothing and nothing names it.
pub struct Orphan;
