use rand::Rng;
use rayon_adaptive::adaptive_sort_raw_logs_with_policies;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use rayon_logs::prelude::*;
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
        .num_threads(3)
        .build()
        .expect("failed");
    let mut v = random_vec(100_000_000);
    let log = pool.logging_install(|| adaptive_sort_raw_logs_with_policies(&mut v, Policy::Join3(2_000_000), Default::default())).1;
    log.save_svg("join3.svg")
        .expect("saving svg file failed");
}
