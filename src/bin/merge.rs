use rayon_adaptive::adaptive_sort_raw_with_policies;
use rayon_adaptive::Policy;
use std::env;

fn main() {
    let size: u32 = 100_000_000;
    let mut inverted: Vec<u32> = (0..size).rev().collect();
    adaptive_sort_raw_with_policies(&mut inverted, Policy::Join3(1000), Policy::DefaultPolicy);
    assert_eq!(inverted, (0..size).collect::<Vec<u32>>());
}
