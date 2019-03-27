use rayon_adaptive::adaptive_sort_raw_with_policies;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use rayon_logs::ThreadPoolBuilder;

fn main() {
    let pool = rayon_logs::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .expect("failed");
    pool.compare()
        .attach_algorithm_with_setup(
            "raw_pol_JC_adapt",
            || (0..50_000_000).collect::<Vec<u32>>(),
            |mut v| {
                adaptive_sort_raw_with_policies(
                    &mut v,
                    Policy::JoinContext(3_125_000),
                    Default::default(),
                );
                v
            },
        )
        .generate_logs("compare.html")
        .expect("saving failed");
}
