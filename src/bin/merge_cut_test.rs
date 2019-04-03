use rand::Rng;
use rayon_adaptive::adaptive_sort_raw_cut_with_policies;
use rayon_adaptive::Policy;
use thread_binder::ThreadPoolBuilder;

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
    // let size = 1_000_000;
    let size = 8194; //10_000;
    let threads = 4;
    // let policies = vec![Policy::Join(1000), Policy::JoinContext(1000)];
    let policies = vec![Policy::Join(1000)];

    let mut v = random_vec(size);

    for sort_policy in policies.iter() {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("Failed to build the pool");
        pool.install(|| {
            adaptive_sort_raw_cut_with_policies(&mut v, *sort_policy, Policy::DefaultPolicy)
        });
    }
}
