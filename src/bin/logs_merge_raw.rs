use rand::Rng;
use rayon_adaptive::adaptive_sort_raw_logs_with_policies;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use rayon_logs::ThreadPoolBuilder;

/// Return a random vector of values between a min and a max values
fn random_vec_with_range(size: usize, min_value: u32, max_value: u32) -> Vec<u32> {
    (0..size)
        .map(|_| rand::thread_rng().gen_range(min_value, max_value))
        .collect()
}

/// Return a random vector
fn random_vec(size: usize) -> Vec<u32> {
    random_vec_with_range(size, 0, size as u32)
}

fn main() {
    let pool = rayon_logs::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("failed");
    pool.compare()
        .runs_number(1)
        .attach_algorithm_with_setup(
            "JC Reversed",
            || (0..50_000_000).rev().collect::<Vec<u32>>(),
            |mut v| {
                adaptive_sort_raw_logs_with_policies(
                    &mut v,
                    Policy::JoinContext(3_125_000),
                    Default::default(),
                );
                v
            },
        )
        .attach_algorithm_with_setup(
            "JC Sorted",
            || (0..50_000_000).collect::<Vec<u32>>(),
            |mut v| {
                adaptive_sort_raw_logs_with_policies(
                    &mut v,
                    Policy::JoinContext(3_125_000),
                    Default::default(),
                );
                v
            },
        )
        .attach_algorithm_with_setup(
            "JC Random",
            || random_vec(50_000_000),
            |mut v| {
                adaptive_sort_raw_logs_with_policies(
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
