//! This crate provides mechanisms for designing adaptive algorithms for rayon.
#![type_length_limit = "2097152"]
#![warn(clippy::all)]
#![deny(missing_docs)]

#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

/// Divisibility traits and implementations
pub(crate) mod divisibility;
pub use divisibility::{BasicPower, BlockedPower, IndexedPower};
/// Adaptive iterators
pub mod iter;
/// Import all traits in prelude to enable adaptive iterators.
pub mod prelude;
/// Different available scheduling policies.
#[derive(Debug, Clone, Copy)]
pub enum Policy {
    /// Use rayon's scheduling algorithm.
    Rayon(usize),
    /// Split recursively until given size is reached.
    Join(usize),
    /// Split adaptively according to steal requests.
    /// Local iterator sizes are between given sizes.
    Adaptive(usize, usize),
    /// Just run sequentially
    Sequential,
}
/// All scheduling algorithms.
pub(crate) mod schedulers;

pub(crate) mod smallchannel;
pub(crate) mod utils;

/// Algorithms
#[macro_use]
pub use algorithms::merging_algorithms::*;

/// Algorithms
pub(crate) mod algorithms;
pub use algorithms::merge2join::adaptive_sort_join2;
pub use algorithms::merge3_by_2_join::adaptive_sort_join3_by_2;
pub use algorithms::merge3join::adaptive_sort_join3;
pub use algorithms::merge3join_2_buffers::adaptive_sort_join3_2_buffers;
pub use algorithms::merge3join_context_2_buffers::adaptive_sort_join_context3;
pub use algorithms::merge3join_no_copy::adaptive_sort_join3_no_copy;
pub use algorithms::merge3join_swap::adaptive_sort_join3_swap;
pub use algorithms::merge_join_context_2_buffers::adaptive_sort_join_context_join;
