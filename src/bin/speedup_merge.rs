use itertools::iproduct;
use rand::Rng;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::adaptive_sort_with_policies;
use rayon_adaptive::Policy;
use std::iter::once;
use std::iter::repeat_with;
use time::precise_time_ns;

/// Return a random vector of values between a min and a max values
fn random_vec_with_range(size: usize, min_value: u32, max_value: u32) -> Vec<u32> {
    (0..size)
        .map(|_| rand::thread_rng().gen_range(min_value, max_value))
        .collect()
}

// -------------------------------  Functions to generate vectors ------------------------------------

/// Return a sorted vec of size size
fn sorted_vec(size: usize) -> Vec<u32> {
    (0..size).map(|x| x as u32).collect()
}

/// Return a reversed vec of size size
fn reversed_vec(size: usize) -> Vec<u32> {
    (0..size).rev().map(|x| x as u32).collect()
}

/// Return a random vector with duplicates values
/// (we are going to shrink the range for picking random values)
fn random_vec_with_duplicates(size: usize) -> Vec<u32> {
    random_vec_with_range(size, 0, size as u32 / 8)
}

/// Return a random vector
fn random_vec(size: usize) -> Vec<u32> {
    random_vec_with_range(size, 0, size as u32)
}

// -------------------------------  Functions to make measures ------------------------------------
//

fn bench<INPUT, I, F>(init_fn: I, timed_fn: F) -> u64
where
    INPUT: Sized,
    I: Fn() -> INPUT,
    F: Fn(INPUT),
{
    let input = init_fn();
    let begin_time: u64 = precise_time_ns();
    timed_fn(input);
    let end_time: u64 = precise_time_ns();
    end_time - begin_time
}

/// Measure the execution time of the adaptive sort given the policies
fn measure_adaptive_sort(original_vec: &Vec<u32>, sort_policy: Policy, fuse_policy: Policy) -> u64 {
    let mut v = original_vec.clone();
    let begin_time: u64 = precise_time_ns();
    adaptive_sort_with_policies(&mut v, sort_policy, fuse_policy);
    let end_time: u64 = precise_time_ns();
    end_time - begin_time
}

/// Measure the execution time of the sequential sort
fn measure_sequential_sort(original_vec: &Vec<u32>) -> u64 {
    let mut v = original_vec.clone();
    let begin_time: u64 = precise_time_ns();
    v.sort();
    let end_time: u64 = precise_time_ns();
    end_time - begin_time
}

/// Measure the speedup (time of the adaptive sort / time of the sequential sort)
/// f is the function that will generate the arrays
fn measure_speedup<INPUT: Sized, I: Fn() -> INPUT, F: Fn(INPUT), F2: Fn(INPUT)>(
    init_fn: I,
    seq_fn: F,
    par_fn: F2,
    iterations: usize,
) -> f64 {
    repeat_with(|| {
        let parallel_time = bench(&init_fn, &par_fn);
        let sequential_time = bench(&init_fn, &seq_fn);
        parallel_time as f64 / (sequential_time as f64 * iterations as f64)
    })
    .take(iterations)
    .sum()
}

fn speedup_by_processors<
    INPUT: Sized,
    I: Fn() -> INPUT + Send + Copy + Sync,
    F: Fn(INPUT) + Send + Copy + Sync,
    F2: Fn(INPUT) + Send + Copy + Sync,
    THREADS: IntoIterator<Item = usize>,
>(
    init_fn: I,
    seq_fn: F,
    par_fn: F2,
    iterations: usize,
    threads_numbers: THREADS,
) -> Vec<(usize, f64)> {
    threads_numbers
        .into_iter()
        .map(|threads| {
            let pool = ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("building pool failed");
            (
                threads,
                pool.install(|| measure_speedup(init_fn, seq_fn, par_fn, iterations)),
            )
        })
        .collect()
}

fn main() {
    let iterations = 100;
    let threads: Vec<usize> = (1..3).collect();
    let policies = vec![Policy::Join(1000), Policy::JoinContext(1000)];
    let input_generators = vec![
        (Box::new(random_vec) as Box<Fn(usize) -> Vec<u32>>, "random"),
        (Box::new(sorted_vec) as Box<Fn(usize) -> Vec<u32>>, "sorted"),
        (
            Box::new(reversed_vec) as Box<Fn(usize) -> Vec<u32>>,
            "reversed",
        ),
    ];
    let algorithms: Vec<_> = once((
        Box::new(|mut v: Vec<u32>| v.sort()) as Box<Fn(Vec<u32>)>,
        "sequential".into(),
    ))
    .chain(
        iproduct!(policies.clone(), policies).map(|(sort_policy, fuse_policy)| {
            (
                Box::new(move |mut v: Vec<u32>| {
                    adaptive_sort_with_policies(&mut v, sort_policy, fuse_policy)
                }) as Box<Fn(Vec<u32>)>,
                format!("{:?}/{:?}", sort_policy, fuse_policy),
            )
        }),
    )
    .collect();
}
