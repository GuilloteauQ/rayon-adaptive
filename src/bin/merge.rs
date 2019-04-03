use rayon_adaptive::adaptive_sort_raw_cut;
use std::env;

fn main() {
    let size: u32 = 1_000_000;
    let mut inverted: Vec<u32> = (0..size).rev().collect();
    adaptive_sort_raw_cut(&mut inverted);
    assert_eq!(inverted, (0..size).collect::<Vec<u32>>());
}
