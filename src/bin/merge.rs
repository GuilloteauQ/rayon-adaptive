use rayon_adaptive::adaptive_sort_raw;
use std::env;

fn main() {
    let size: u32 = 100_000_000;
    let mut inverted: Vec<u32> = (0..size).rev().collect();
    adaptive_sort_raw(&mut inverted);
    assert_eq!(inverted, (0..size).collect::<Vec<u32>>());
}
