use itertools::{iproduct, Itertools};
use rand::Rng;
use rayon_adaptive::adaptive_sort_raw_with_policies_swap_blocks;
use rayon_adaptive::Policy;
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

fn bench<INPUT, I, F>(init_fn: I, timed_fn: F) -> (u64, bool)
where
    INPUT: Sized,
    I: Fn() -> INPUT,
    F: Fn(INPUT) -> bool,
{
    let input = init_fn();
    let begin_time: u64 = precise_time_ns();
    let state = timed_fn(input);
    let end_time: u64 = precise_time_ns();
    (end_time - begin_time, state)
}

fn tuple_to_vec<A: Clone, B: Clone>(v: &Vec<(A, B)>) -> (Vec<A>, Vec<B>) {
    let mut va = Vec::new();
    let mut vb = Vec::new();
    for (a, b) in v.iter() {
        va.push(a.clone());
        vb.push(b.clone());
    }
    (va, vb)
}

/// Measure the speedup (time of the adaptive sort / time of the sequential sort)
/// f is the function that will generate the arrays
fn average_times<INPUT: Sized, I: Fn() -> INPUT, F: Fn(INPUT) -> bool>(
    init_fn: I,
    timed_fn: F,
    iterations: usize,
) -> (f64, usize) {
    let mut ve = Vec::new();
    for _ in 0..iterations {
        ve.push(bench(&init_fn, &timed_fn));
    }
    let v = tuple_to_vec(&ve);
    let time: f64 = v.0.iter().sum::<u64>() as f64 / iterations as f64;
    let states: usize = v.1.iter().map(|&b| if b { 1 } else { 0 }).sum();
    (time, states)
}

fn times_by_processors<
    INPUT: Sized,
    I: Fn() -> INPUT + Send + Copy + Sync,
    F: Fn(INPUT) -> bool + Send + Copy + Sync,
    THREADS: IntoIterator<Item = usize>,
>(
    init_fn: I,
    timed_fn: F,
    iterations: usize,
    threads_numbers: THREADS,
) -> Vec<(f64, usize)> {
    threads_numbers
        .into_iter()
        .map(move |threads| {
            let pool = ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("building pool failed");
            pool.install(|| average_times(init_fn, timed_fn, iterations))
        })
        .collect()
}

fn main() {
    let iterations = 100;
    let sizes = vec![1_000_000];
    let threads: Vec<usize> = vec![4];
    let policies = vec![Policy::Join(1000), Policy::JoinContext(1000)];
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
        (
            Box::new(random_vec_with_duplicates) as Box<Fn(usize) -> Vec<u32> + Sync>,
            "random_with_duplicates",
        ),
    ];
    let algorithms: Vec<_> = iproduct!(policies.clone(), policies.clone())
        .map(|(sort_policy, fuse_policy)| {
            (
                Box::new(move |mut v: Vec<u32>| {
                    adaptive_sort_raw_with_policies_swap_blocks(&mut v, sort_policy, fuse_policy)
                }) as Box<Fn(Vec<u32>) -> bool + Sync + Send>,
                format!("Swap {:?}/{:?}", sort_policy, fuse_policy),
            )
        })
        .collect();

    for (generator_f, generator_name) in input_generators.iter() {
        println!(">>> {}", generator_name);
        for size in sizes.iter() {
            println!("Size: {}", size);
            let algo_results: Vec<_> = algorithms
                .iter()
                .map(|(algo_f, name)| {
                    print!("{}: ", name);
                    let res = times_by_processors(
                        || generator_f(*size),
                        |v| algo_f(v),
                        iterations,
                        threads.clone(),
                    );
                    print!(
                        "{}/{} copies\n",
                        res.iter().map(|(_, b)| b).sum::<usize>(),
                        iterations
                    );
                    res
                })
                .collect();
        }
    }
}
