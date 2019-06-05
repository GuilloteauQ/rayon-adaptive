//! adaptive parallel merge sort.
use rayon_adaptive::adaptive_sort_join_context_join;
// use rayon_adaptive::Policy;
use rand::seq::SliceRandom;
use rand::thread_rng;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::prelude::*;

#[cfg(feature = "logs")]
use rayon_logs::subgraph;

fn main() {
    let size = 50_000_000;
    let block_size = 3_125_000;
    let block_size_fuse = 10;
    let v: Vec<u32> = (0..size).collect();
    let mut shuffled: Vec<u32> = (0..size).collect();

    let mut rng = thread_rng();
    shuffled.shuffle(&mut rng);

    let mut inverted: Vec<u32> = (0..size).rev().collect();
    #[cfg(feature = "logs")]
    {
        let pool = rayon_logs::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .expect("failed");
        let (_, log) = pool.logging_install(|| {
            adaptive_sort_join_context_join(&mut inverted, block_size, block_size_fuse)
        });

        log.save_svg("merge_sort_join_context_join_par_fuse.svg")
            .expect("saving svg file failed");
    }
    #[cfg(not(feature = "logs"))]
    {
        adaptive_sort_join_context_join(&mut inverted, block_size, block_size_fuse);
    }
    assert_eq!(v, inverted);
}