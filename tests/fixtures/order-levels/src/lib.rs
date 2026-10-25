//! Levels, reach, and seen order.

/// The main entry: it reaches the most declarations.
pub struct Run {
    pub config: Config,
    pub steps: Vec<Step>,
}

/// A root that ties Report on reach and comes first by path.
pub struct Able {
    pub leaf: Leaf,
}

/// A root that ties Able on reach.
pub struct Report {
    pub summary: Summary,
}

/// Named by Run at level 1 and by Step at level 2, so it lands at level 3.
pub struct Config {
    pub limit: u32,
}

/// Named by Run; first names Trace, then Log.
pub struct Step {
    pub a_trace: Trace,
    pub b_log: Log,
    pub config: Config,
}

/// Named only by Step.
pub struct Log;

/// Named only by Step, before Log.
pub struct Trace;

/// Named only by Able.
pub struct Leaf;

/// Named only by Report.
pub struct Summary;

/// Names nothing and nothing names it.
pub struct Orphan;
