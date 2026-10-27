//! Groups of types by module.

/// The only root: it names Core, Db, and Log, in that order.
pub struct App {
    pub a_core: engine::Core,
    pub b_db: store::Db,
    pub c_log: Log,
}

/// Named only by App, in the crate root.
pub struct Log {
    pub level: Level,
}

/// Named only by Log, in the crate root.
pub struct Level;

pub mod engine {
    //! The engine.

    /// Named only by App; names Plan, then Queue.
    pub struct Core {
        pub a_plan: Plan,
        pub b_queue: sched::Queue,
    }

    /// Named only by Core.
    pub struct Plan;

    pub mod sched {
        //! The scheduler.

        /// Named only by Core; names Slot.
        pub struct Queue {
            pub slot: deep::Slot,
        }

        pub mod deep {
            //! Three segments deep: its group is named for the first two.

            /// Named only by Queue.
            pub struct Slot;
        }
    }
}

pub mod store {
    //! The store.

    /// Named only by App: a lone type in its module.
    pub struct Db;
}

pub mod util {
    //! Helpers nothing uses.

    /// Names nothing and nothing names it.
    pub struct Spare;
}
