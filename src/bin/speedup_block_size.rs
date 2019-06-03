use itertools::{iproduct, Itertools};
use rand::Rng;
use rayon::prelude::*;
use rayon_adaptive::{
    adaptive_sort_join2, adaptive_sort_join3, adaptive_sort_join3_by_2,
    adaptive_sort_join3_no_copy, adaptive_sort_join3_swap,
};
use std::fs::File;
use std::io::prelude::*;
use std::iter::{once, repeat_with};
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
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("building pool failed");
        pool.install(|| average_times(init_fn, timed_fn, iterations))
    })
}

fn main() {
    let iterations = 10;
    let size = 10_000_000;
    // let block_sizes = vec![50_000, 100_000, 200_000, 500_000, 1_000_000];
    let block_sizes: Vec<usize> = (1..size)
        .map(|r| (size as f64 / ((2 * r) as f64).powi(3)).ceil() as usize + 1)
        .take_while(|&x| x > size / (100 * 1_000) && x < size / 3)
        .collect();
    // let block_sizes: Vec<usize> = (1..100).step_by(10).map(|r| size * r / 100).collect();

    println!("{:?}", block_sizes);

    // let threads: Vec<usize> = (1..33).collect();
    let threads = vec![1, 2, 3, 4, 6, 8, 16, 32];
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
        /*
        (
            Box::new(random_vec_with_duplicates) as Box<Fn(usize) -> Vec<u32> + Sync>,
            "random_with_duplicates",
        ),
        */
    ];

    let seq_algo = Box::new(|mut v: Vec<u32>| v.sort()) as Box<Fn(Vec<u32>) + Sync + Send>;
    let algorithms: Vec<_> = once((
        Box::new(move |mut v: Vec<u32>, block_size: usize| {
            adaptive_sort_join3(&mut v, block_size, block_size)
        }) as Box<Fn(Vec<u32>, usize) + Sync + Send>,
        format!("3/3-Join"),
    ))
    .collect();

    for (generator_f, generator_name) in input_generators.iter() {
        println!(">>> {}", generator_name);
        let mut file =
            File::create(format!("new_speedup_block_size_{}.dat", generator_name)).unwrap();
        write!(&mut file, "#size threads block_size").expect("failed writing to file");
        writeln!(
            &mut file,
            "{}",
            algorithms.iter().map(|(_, label)| label).join(" ")
        )
        .expect("failed writing to file");

        let mut seq_time = 0;

        for _ in 0..iterations {
            seq_time += bench(|| generator_f(size), |mut v: Vec<u32>| v.sort());
        }
        seq_time /= iterations as u64;

        let mut algo_results: Vec<Vec<_>> = Vec::new();

        for block_size in block_sizes.iter() {
            println!("Block Size: {}", block_size);
            algo_results.push(
                algorithms
                    .iter()
                    .map(|(algo_f, _)| {
                        times_by_processors(
                            || generator_f(size),
                            |v| algo_f(v, *block_size),
                            iterations,
                            threads.clone(),
                        )
                        .collect::<Vec<_>>()
                    })
                    .collect(),
            );
            // for (index, threads_number) in threads.iter().enumerate() {
            //     writeln!(
            //         &mut file,
            //         "{}",
            //         once(threads_number.to_string())
            //             .chain(once(block_size.to_string()))
            //             .chain(algo_results.iter().map(|v| v[index].to_string()))
            //             .chain(once(size.to_string()))
            //             .chain(once(seq_time.to_string()))
            //             .join(" ")
            //     )
            //     .expect("failed writing");
            // }
        }
        for (index, threads_number) in threads.iter().enumerate() {
            for (index_block_size, block_size) in block_sizes.iter().enumerate() {
                writeln!(
                    &mut file,
                    "{}",
                    once(threads_number.to_string())
                        .chain(once(block_size.to_string()))
                        .chain(
                            algo_results[index_block_size]
                                .iter()
                                .map(|v| v[index].to_string())
                        )
                        .chain(once(size.to_string()))
                        .chain(once(seq_time.to_string()))
                        .join(" ")
                )
                .expect("failed writing");
            }
        }
    }
}
