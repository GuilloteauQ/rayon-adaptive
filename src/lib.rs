//! This crate provides mechanisms for designing adaptive algorithms for rayon.
//#![allow(unknown_lints)]
#![type_length_limit = "2097152"]
#![warn(clippy::all)]
#[cfg(not(feature = "logs"))]
extern crate rayon;
#[cfg(feature = "logs")]
extern crate rayon as real_rayon;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
#[macro_use]
extern crate smallvec;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
mod traits;
pub use crate::traits::*;
mod scheduling;
pub mod utils;
pub use crate::utils::fuse_slices;
mod slices;
pub use crate::slices::{EdibleSlice, EdibleSliceMut};
mod activated_input;
mod chunks;
pub mod iter;
pub use crate::iter::hash::{par_elements, par_iter, par_keys};
pub use crate::iter::iter::Iter;
pub use crate::iter::map::Map;
pub use crate::iter::zip::Zip;

mod folders;
pub use crate::folders::Folder;
mod policy;
pub use crate::policy::Policy;
pub mod atomiclist;
pub mod prelude;
pub mod smallchannel;
pub use smallchannel::{small_channel, SmallReceiver, SmallSender};

mod algorithms;
pub use crate::algorithms::infix_solvers::*;
pub use crate::algorithms::prefix::adaptive_prefix;
pub use crate::algorithms::{
    merge_sort::adaptive_sort, merge_sort::adaptive_sort_with_policies,
    merge_sort_no_copy::adaptive_sort_no_copy_with_policies, merge_sort_raw::adaptive_sort_raw,
    merge_sort_raw::adaptive_sort_raw_with_policies, merge_sort_raw_cut::adaptive_sort_raw_cut,
    merge_sort_raw_cut::adaptive_sort_raw_cut_with_policies,
    merge_sort_raw_cut_ceil::adaptive_sort_raw_cut_ceil_with_policies,
    merge_sort_raw_cut_floor::adaptive_sort_raw_cut_floor_with_policies,
    merge_sort_raw_logs::adaptive_sort_raw_logs_with_policies,
    merge_sort_swap_blocks::adaptive_sort_raw_swap_blocks,
    merge_sort_swap_blocks::adaptive_sort_raw_with_policies_swap_blocks,
};

/// Execute potentially `oper_a` and `oper_b` in parallel like in a standard join.
/// Then the last closure to finish calls `oper_c` on both results.
pub fn depjoin<A, B, C, RA, RB, RC>(oper_a: A, oper_b: B, oper_c: C) -> RC
where
    A: FnOnce() -> RA + Send,
    B: FnOnce() -> RB + Send,
    C: FnOnce(RA, RB) -> RC + Send,
    RA: Send,
    RB: Send,
    RC: Send,
{
    let done = &AtomicBool::new(false);
    let (sender_a, receiver_a) = small_channel();
    let (sender_b, receiver_b) = small_channel();
    let results = rayon::join(
        move || {
            let ra = oper_a();
            let we_are_last = done.swap(true, Ordering::SeqCst);
            if we_are_last {
                let rb = receiver_a.recv().expect("receiving result failed");
                Some(oper_c(ra, rb))
            } else {
                sender_b.send((ra, oper_c));
                None
            }
        },
        move || {
            let rb = oper_b();
            let we_are_last = done.swap(true, Ordering::SeqCst);
            if we_are_last {
                let (ra, oper_c) = receiver_b.recv().expect("receiving result failed");
                Some(oper_c(ra, rb))
            } else {
                sender_a.send(rb);
                None
            }
        },
    );
    results.0.or(results.1).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let s: u64 = depjoin(
            || (1..100_000).sum(),
            || (1..1000).sum(),
            |sa: u64, sb: u64| sa + sb,
        );
        assert_eq!(s, 5000449500);
        let s: u64 = depjoin(
            || (1..1000).sum(),
            || (1..100_000).sum(),
            |sa: u64, sb: u64| sa + sb,
        );
        assert_eq!(s, 5000449500);
    }
}
