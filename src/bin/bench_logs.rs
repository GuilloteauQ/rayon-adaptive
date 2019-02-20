#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::Policy;
use rayon_adaptive::{adaptive_sort, adaptive_sort_with_policies};
use std::env;
extern crate rand;
use rand::Rng;

fn random_vec(size: usize, min_value: u32, max_value: u32) -> Vec<u32> {
    (0..size)
        .map(|_| rand::thread_rng().gen_range(min_value, max_value))
        .collect()
}

fn logs_from_random(
    threads_nb: usize,
    runs: usize,
    size: usize,
    min_value: u32,
    max_value: u32,
    output_filename: &str,
) {
    let join_block_size = 1000;
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads_nb)
        .build()
        .expect("Failed to build the pool");

    pool.compare()
        .runs_number(runs)
        .attach_algorithm_with_setup(
            "Classical Adaptive Sort",
            || random_vec(size, min_value, max_value),
            |mut vec| adaptive_sort(&mut vec),
        )
        .attach_algorithm_with_setup(
            "Adaptive Sort Join Join",
            || random_vec(size, min_value, max_value),
            |mut vec| {
                adaptive_sort_with_policies(
                    &mut vec,
                    Policy::Join(join_block_size),
                    Policy::Join(join_block_size),
                )
            },
        )
        .attach_algorithm_with_setup(
            "Adaptive Sort JoinContext Join",
            || random_vec(size, min_value, max_value),
            |mut vec| {
                adaptive_sort_with_policies(
                    &mut vec,
                    Policy::JoinContext(join_block_size),
                    Policy::Join(join_block_size),
                )
            },
        )
        .attach_algorithm_with_setup(
            "Adaptive Sort Adaptive Join",
            || random_vec(size, min_value, max_value),
            |mut vec| {
                adaptive_sort_with_policies(
                    &mut vec,
                    Policy::Adaptive(join_block_size, 2 * join_block_size),
                    Policy::Join(join_block_size),
                )
            },
        )
        .generate_logs(output_filename)
        .expect("Could not generate logs");
}
fn logs_with_vec(threads_nb: usize, runs: usize, original_vec: &Vec<u32>, output_filename: &str) {
    let join_block_size = 1000;
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads_nb)
        .build()
        .expect("Failed to build the pool");

    pool.compare()
        .runs_number(runs)
        .attach_algorithm("Classical Adaptive Sort", || {
            let mut v = original_vec.clone();
            adaptive_sort(&mut v)
        })
        .attach_algorithm("Adaptive Sort Join Join", || {
            let mut v = original_vec.clone();
            adaptive_sort_with_policies(
                &mut v,
                Policy::Join(join_block_size),
                Policy::Join(join_block_size),
            )
        })
        .attach_algorithm("Adaptive Sort JoinContext Join", || {
            let mut v = original_vec.clone();
            adaptive_sort_with_policies(
                &mut v,
                Policy::JoinContext(join_block_size),
                Policy::Join(join_block_size),
            )
        })
        .attach_algorithm("Adaptive Sort Adaptive Join", || {
            let mut v = original_vec.clone();
            adaptive_sort_with_policies(
                &mut v,
                Policy::Adaptive(join_block_size, 2 * join_block_size),
                Policy::Join(join_block_size),
            )
        })
        .generate_logs(output_filename)
        .expect("Could not generate logs");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: cargo run --bin bench_logs SIZE NUM_THREADS");
        return;
    }

    let slice_size: u32 = args[1].parse::<u32>().unwrap();
    let num_threads: usize = args[2].parse::<usize>().unwrap();
    let runs = 100;

    // Reversed Vector
    let inverted: Vec<u32> = (0..slice_size).rev().collect();
    logs_with_vec(num_threads, runs, &inverted, "logs/inverted.html");
    println!(">>> Finished logs for a reversed vector");

    // Already Sorted Vector
    let sorted: Vec<u32> = (0..slice_size).collect();
    logs_with_vec(num_threads, runs, &sorted, "logs/sorted.html");
    println!(">>> Finished logs for a sorted vector");

    // Random Vector
    logs_from_random(
        num_threads,
        runs,
        slice_size as usize,
        0,
        slice_size * 2,
        "logs/random.html",
    );
    println!(">>> Finished logs for a random vector");

    // Random Vector with duplicates
    logs_from_random(
        num_threads,
        runs,
        slice_size as usize,
        0,
        256,
        "logs/duplicates.html",
    );
    println!(">>> Finished logs for a random vector with duplicates");
}
