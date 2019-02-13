#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::adaptive_sort;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let size: u32 = args[1].parse::<u32>().unwrap();
    let block_size: usize = args[2].parse::<usize>().unwrap();
    let mut inverted: Vec<u32> = (0..size).rev().collect();
    {
        let pool = ThreadPoolBuilder::new()
            .num_threads(args[3].parse::<usize>().unwrap())
            .build()
            .expect("building pool failed");
        let (_, log) = pool.logging_install(|| adaptive_sort(&mut inverted, block_size));

        log.save_svg("quentin_merge.svg").expect("saving svg file failed");
        println!("the SVG file has been generated");
    }
    // println!("{:?}", inverted);
}
