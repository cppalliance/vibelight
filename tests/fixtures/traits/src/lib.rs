//! Traits.

/// Watches a run.
pub trait Observer: Send + Sync {
    /// The item type.
    type Item: Clone;

    /// The most items kept.
    const LIMIT: usize = 64;

    /// Records one event.
    fn record(&self, event: &str);

    /// Clears the buffer.
    fn flush(&mut self) {}

    /// Ends the run.
    fn finish(self) -> u32
    where
        Self: Sized;

    /// Makes a default observer.
    fn make() -> Self
    where
        Self: Sized;
}

/// A marker that is unsafe to implement.
pub unsafe trait Zeroable {}

/// A generic trait with a where clause.
pub trait Convert<T>
where
    T: Clone,
{
    /// Converts a value.
    fn convert(&self, value: T) -> T;
}

/// A trait with a generic associated type.
pub trait Lender {
    /// The borrowed item.
    type Loan<'a>
    where
        Self: 'a;

    /// Lends an item.
    fn lend(&mut self) -> Self::Loan<'_>;
}
