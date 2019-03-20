use itertools::{iproduct, Itertools};
use rand::Rng;
use rayon_adaptive::Policy;
use rayon_adaptive::{
    adaptive_sort_no_copy_with_policies, adaptive_sort_raw_with_policies,
    adaptive_sort_raw_with_policies_swap_blocks, adaptive_sort_with_policies,
};
use std::fs::File;
use std::io::prelude::*;
use std::iter::{once, repeat_with};
use thread_binder::ThreadPoolBuilder;
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

/// Measure the speedup (time of the adaptive sort / time of the sequential sort)
/// f is the function that will generate the arrays
fn average_times<INPUT: Sized, I: Fn() -> INPUT, F: Fn(INPUT)>(
    init_fn: I,
    timed_fn: F,
    iterations: usize,
) -> f64 {
    repeat_with(|| bench(&init_fn, &timed_fn))
        .take(iterations)
        .sum::<u64>() as f64
        / iterations as f64
}

fn times_by_processors<
    INPUT: Sized,
    I: Fn() -> INPUT + Send + Copy + Sync,
    F: Fn(INPUT) + Send + Copy + Sync,
    THREADS: IntoIterator<Item = usize>,
>(
    init_fn: I,
    timed_fn: F,
    iterations: usize,
    threads_numbers: THREADS,
) -> impl Iterator<Item = f64> {
    threads_numbers.into_iter().map(move |threads| {
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("building pool failed");
        pool.install(|| average_times(init_fn, timed_fn, iterations))
    })
}

fn main() {
    let iterations = 10;
    let size = 5_000;
    let threads: Vec<usize> = (1..5).collect();
    let policies = vec![Policy::Join(100), Policy::JoinContext(100)];
    let input_generators = vec![
        (
            Box::new(random_vec) as Box<Fn(usize) -> Vec<u32> + Sync>,
            "random",
        ),
        (
            Box::new(sorted_vec) as Box<Fn(usize) -> Vec<u32> + Sync>,
            "sorted",
        ),
        (
            Box::new(reversed_vec) as Box<Fn(usize) -> Vec<u32> + Sync>,
            "reversed",
        ),
    ];

    let algorithm_versions: Vec<_> = vec![
        (
            Box::new(
                move |mut v: Vec<u32>, sort_policy: Policy, fuse_policy: Policy| {
                    adaptive_sort_with_policies(&mut v, sort_policy, fuse_policy)
                },
            ) as Box<Fn(Vec<u32>, Policy, Policy) + Sync + Send>,
            "Classic",
        ),
        (
            Box::new(
                move |mut v: Vec<u32>, sort_policy: Policy, fuse_policy: Policy| {
                    adaptive_sort_raw_with_policies(&mut v, sort_policy, fuse_policy)
                },
            ) as Box<Fn(Vec<u32>, Policy, Policy) + Sync + Send>,
            "Raw",
        ),
        (
            Box::new(
                move |mut v: Vec<u32>, sort_policy: Policy, fuse_policy: Policy| {
                    adaptive_sort_no_copy_with_policies(&mut v, sort_policy, fuse_policy)
                },
            ) as Box<Fn(Vec<u32>, Policy, Policy) + Sync + Send>,
            "No copy",
        ),
    ];

    for (generator_f, generator_name) in input_generators.iter() {
        for sort_policy in policies.iter() {
            let results: Vec<_> = algorithm_versions
                .iter()
                .map(|(algo_template, _)| {
                    times_by_processors(
                        || generator_f(size),
                        |mut v: Vec<u32>| algo_template(v, *sort_policy, Policy::DefaultPolicy),
                        iterations,
                        threads.clone(),
                    )
                    .collect::<Vec<_>>()
                })
                .collect();
            let mut file = File::create(format!(
                "data/comp_{}_{:?}.dat",
                generator_name, sort_policy
            ))
            .unwrap();
            write!(&mut file, "#size threads ").expect("failed writing to file");
            writeln!(
                &mut file,
                "{}",
                algorithm_versions.iter().map(|(_, label)| label).join(" ")
            )
            .expect("failed writing to file");
            for (index, threads_number) in threads.iter().enumerate() {
                writeln!(
                    &mut file,
                    "{}",
                    once(size.to_string())
                        .chain(once(threads_number.to_string()))
                        .chain(results.iter().map(|v: &Vec<f64>| v[index].to_string()))
                        .join(" ")
                )
                .expect("failed writing");
            }
        }
    }
}
