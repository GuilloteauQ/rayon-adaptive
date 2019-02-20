use rayon_adaptive::adaptive_sort_with_policies;
use rayon_adaptive::Policy;
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
fn measure_speedup(
    f: &Box<Fn(usize) -> Vec<u32>>,
    iterations: usize,
    vec_size: usize,
    sort_policy: Policy,
    fuse_policy: Policy,
    threads_num: usize,
) -> f64 {
    // ----- Creation of a pool with the given number of threads -----
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads_num)
        .build()
        .expect("Failed to build the pool");
    let mut times_adaptive_sort: Vec<u64> = Vec::with_capacity(iterations);
    let mut times_sequential_sort: Vec<u64> = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let v = f(vec_size);
        times_adaptive_sort
            .push(pool.install(|| measure_adaptive_sort(&v, sort_policy, fuse_policy)));
        times_sequential_sort.push(pool.install(|| measure_sequential_sort(&v)));
    }
    compute_mean(times_adaptive_sort, times_sequential_sort)
}

/// Computes the mean
fn compute_mean(t1: Vec<u64>, t2: Vec<u64>) -> f64 {
    let size = t1.len() as f64;
    t1.iter()
        .zip(t2.iter())
        .map(|(&a, &b)| (a as f64) / (b as f64 * size))
        .sum()
}

/// Returns a vec of pair with the number of threads and the speedup
fn speedup_by_processors(
    procs_range: &Vec<usize>,
    f: Box<Fn(usize) -> Vec<u32>>,
    iterations: usize,
    vec_size: usize,
    sort_policy: Policy,
    fuse_policy: Policy,
) -> Vec<(usize, f64)> {
    procs_range
        .iter()
        .map(|&threads_num| {
            (
                threads_num,
                measure_speedup(&f, iterations, vec_size, sort_policy, fuse_policy, threads_num),
            )
        })
        .collect()
}

fn bench_speedup<S: AsRef<str>>(f: Box<Fn(usize) -> Vec<u32>>, label: S) {
    // TODO: zip everything and plot data
    let procs: Vec<usize> = (1..3).collect();
    let sorting_policies: Vec<Policy> = vec![Policy::Join(1000), Policy::JoinContext(1000)];
    let fusing_policies: Vec<Policy> = vec![Policy::Join(1000), Policy::JoinContext(1000)];
    let sizes: Vec<usize> = vec![5000, 10000, 25000, 50000, 100000];

    let results: Vec<(usize, f64)> = speedup_by_processors(&procs, f, 100, sizes[0], sorting_policies[0], fusing_policies[0]);
    println!("{}: {:?}", label.as_ref(), results);
}

fn main() {
    bench_speedup(Box::new(random_vec), "Random");
    bench_speedup(Box::new(sorted_vec), "Sorted");
    bench_speedup(Box::new(reversed_vec), "Reversed");
    bench_speedup(Box::new(random_vec_with_duplicates), "Random with duplicates");
}
