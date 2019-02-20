use rayon_adaptive::adaptive_sort_with_policies;
use std::env;
use crate::Policy;
extern crate time;
use time::precise_time_ns;
extern crate rand;
use rand::Rng;

/// Return a random vector of values between a min and a max values
fn random_vec_with_range(size: usize, min_value: u32, max_value: u32) -> Vec<u32> {
    (0..size)
        .map(|_| rand::thread_rng().gen_range(min_value, max_value))
        .collect()
}

// -------------------------------  Functions to generate vectors ------------------------------------

/// Return a sorted vec of size size
fn sorted_vec(size: usize) -> Vec<u32> {
    (0..size).collect()
}

/// Return a reversed vec of size size
fn reversed_vec(size: usize) -> Vec<u32> {
    (0..size).rev().collect()
}

/// Return a random vector with duplicates values
/// (we are going to shrink the range for picking random values)
fn random_vec_with_duplicates(size: usize) -> Vec<u32> {
    random_vec_with_range(size, 0, size / 8)
}

/// Return a random vector
fn random_vec(size: usize) -> Vec<u32> {
    random_vec_with_range(size, 0, size)
}

// -------------------------------  Functions to make measures ------------------------------------

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
    sort(&mut v);
    let end_time: u64 = precise_time_ns();
    end_time - begin_time
}


/// Measure the speedup (time of the adaptive sort / time of the sequential sort)
/// f is the function that will generate the arrays
fn measure_speedup(f: Fn(usize) -> Vec<u32>, iterations: usize, vec_size: usize, sort_policy: Policy, fuse_policy: Policy, threads_num: usize) -> f64 {
    // ----- Creation of a pool with the given number of threads -----
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads_num).build().expect("Failed to build the pool");
    let mut times_adaptive_sort: Vec<u64> = Vec::with_capacity(iterations);
    let mut times_sequential_sort: Vec<u64> = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let v = f(vec_size);
        times_adaptive_sort.push(pool.install(|| measure_adaptive_sort(&v, sort_policy, fuse_policy)));
        times_sequential_sort.push(pool.install(|| measure_sequential_sort(&v)));
    }
    compute_mean(times_adaptive_sort, times_sequential_sort)
}

/// Computes the mean
fn compute_mean(t1: Vec<u64>, t2: Vec<u64>) -> f64 {
    let size = t1.len();
    t1.iter().zip(t2.iter()).map(|(a, b)| (a as f64) / ((b * size) as f64)).sum()
}


/// Returns a vec of pair with the number of threads and the speedup
fn speedup_by_processors(procs_range: &Vec<usize>, f: Fn(usize) -> Vec<u32>, iterations: usize, vec_size: usize, sort_policy: Policy, fuse_policy: Policy) -> Vec<(usize, f64)> {
    procs_range.iter().map(|threads_num| (threads_num, measure_speedup(f, iterations, vec_size, sort_policy, fuse_policy))).collect()
}

fn main() {

    // Range of the number of processors
    let procs: Vec<usize> = (1..8).collect();
    // Vector of functions for generating the vectors to sort
    let vector_generation_function: Vec<Fn(usize) -> Vec<u32>> = vec![sorted_vec, reversed_vec, random_vec, random_vec_with_duplicates];
    // Vector of SORTING policies
    let sorting_policies: Vec<Policy> = vec![Policy::Join(1000), Policy::JoinContext(1000)];
    // Vector of FUSING policies
    let fusing_policies: Vec<Policy> = vec![Policy::Join(1000), Policy::JoinContext(1000)];


}
