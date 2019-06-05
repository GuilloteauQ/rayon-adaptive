//! adaptive parallel merge sort.
// use crate::divisibility::divisible::{lllllDivisible;
// use crate::schedulers::schedule_join3;
// use crate::utils::merge_2;
// sort related code

use crate::prelude::*;

use crate::algorithms::merging_algorithms::*;

#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

#[cfg(feature = "logs")]
use rayon_logs::subgraph;

macro_rules! fuse_multiple_slices {
    ( $left:expr ) => {
        $left
    };
    ( $left:expr, $($rest:expr),+ ) => {
        {
            let s1 = $left;
            let ptr1 = s1.as_mut_ptr();
            let s2 = fuse_multiple_slices!($($rest),+);
            unsafe {
                assert_eq!(ptr1.add(s1.len()) as *const T, s2.as_ptr());
                std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
            }
        }
    };
}
/// We'll need slices of several vectors at once.
struct SortingSlices<'a, T: 'a> {
    s: Vec<&'a mut [T]>,
    i: usize,
}

impl<'a, T: 'a + Ord + Sync + Copy + Send> SortingSlices<'a, T> {
    /// Call parallel merge on the right slices.
    fn fuse(self, mid: Self, right: Self, block_size_fuse: usize) -> Self {
        let mut left = self;
        let mut mid = mid;
        let mut right = right;
        let depth = left.depth;

        let destination_index = {
            let destination_index = (0..2)
                .find(|&x| x != left.i && x != mid.i && x != right.i)
                .unwrap();

            assert!(left.i == mid.i && mid.i == right.i);
            {
                let left_index = left.i;
                let mid_index = mid.i;
                let right_index = right.i;

                let (left_input, left_output) = left.mut_couple(left_index, destination_index);
                let (mid_input, mid_output) = mid.mut_couple(mid_index, destination_index);
                let (right_input, right_output) = right.mut_couple(right_index, destination_index);

                let output_slice = fuse_multiple_slices!(left_output, mid_output, right_output);
                #[cfg(not(feature = "logs"))]
                merge_3_par(
                    left_input,
                    mid_input,
                    right_input,
                    output_slice,
                    block_size_fuse,
                );

                #[cfg(feature = "logs")]
                //subgraph("Fuse rec master", full_size, || {
                merge_3_par(
                    left_input,
                    mid_input,
                    right_input,
                    output_slice,
                    block_size_fuse,
                );
                //});
            }
            destination_index
        };
        let fused_slices: Vec<_> = left
            .s
            .into_iter()
            .zip(mid.s.into_iter())
            .zip(right.s.into_iter())
            .map(|((left_s, mid_s), right_s)| fuse_multiple_slices!(left_s, mid_s, right_s))
            .collect();
        SortingSlices {
            s: fused_slices,
            i: destination_index,
            split_in_3: true,
            depth: depth.min(right.depth) - 1,
        }
    }

    /// Borrow all mutable slices at once.
    fn mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let (s0, leftover) = self.s.split_first_mut().unwrap();
        let (s1, _) = leftover.split_first_mut().unwrap();
        (s0, s1)
    }
    /// Return the two mutable slices of given indices.
    fn mut_couple(&mut self, i1: usize, i2: usize) -> (&mut [T], &mut [T]) {
        let (s0, s1) = self.mut_slices();
        match (i1, i2) {
            (0, 1) => (s0, s1),
            (1, 0) => (s1, s0),
            _ => panic!("i1 == i2"),
        }
    }
}

impl<'a, T: 'a + Ord + Copy + Sync + Send> Divisible<IndexedPower> for SortingSlices<'a, T> {
    fn base_length(&self) -> Option<usize> {
        self.s[0].base_length()
    }
    fn divide_at(self, i: usize) -> (Self, Self) {
        let v: (Vec<_>, Vec<_>) = self.s.into_iter().map(|s| s.split_at_mut(i)).unzip();
        (
            SortingSlices { s: v.0, i: self.i },
            SortingSlices { s: v.1, i: self.i },
        )
    }
}

/// Parallel sort join 3 to 3
pub fn adaptive_sort_join_context3<T: Ord + Copy + Send + Sync>(
    slice: &mut [T],
    block_size: usize,
    block_size_fuse: usize,
) {
    let slice_len = slice.len();
    let mut tmp_slice1 = Vec::with_capacity(slice.base_length().unwrap());
    unsafe {
        tmp_slice1.set_len(slice.base_length().unwrap());
    }

    let slices = SortingSlices {
        s: vec![slice, tmp_slice1.as_mut_slice()],
        i: 0,
    };

    let recursions = (((slice_len as f64) / (block_size as f64)).log(3.0).ceil() / 2.0 - 0.5).ceil()
        as usize
        * 2;
    let new_block_size =
        { ((slice_len as f64) / (recursions as f64).powf(3.0)).ceil() as usize + 2 };

    #[cfg(not(feature = "logs"))]
    let k = slices.work(|mut slices, size| {
        slices.s[slices.i][0..size].sort();
        slices
    });

    #[cfg(feature = "logs")]
    let k = slices.work(|mut slices, size| {
        subgraph("Sort", slices.s[0].len(), || {
            slices.s[slices.i][0..size].sort();
            slices
        })
    });

    #[cfg(not(feature = "logs"))]
    let mut result_slices = schedule_join_context3(
        k,
        &|l: SortingSlices<T>, m, r| l.fuse(m, r, new_block_size),
        new_block_size,
        recursions,
    );

    #[cfg(feature = "logs")]
    let mut result_slices = schedule_join_context3(
        k,
        &|l: SortingSlices<T>, m, r| {
            subgraph("Fuse", 3 * l.s[0].len(), || l.fuse(m, r, new_block_size))
        },
        new_block_size,
        recursions,
    );

    if result_slices.i != 0 {
        let i = result_slices.i;
        let (destination, source) = result_slices.mut_couple(0, i);
        destination.copy_from_slice(source);
        panic!("Does not go back to slice 0 !");
    }
}
