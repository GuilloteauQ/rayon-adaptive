//! adaptive parallel merge sort.
// use crate::divisibility::divisible::{lllllDivisible;
use crate::schedulers::schedule_join2;
// use crate::utils::merge_2;
// sort related code
use crate::prelude::*;

#[macro_use]
use crate::algorithms::merging_algorithms::*;

#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

#[cfg(feature = "logs")]
use rayon_logs::subgraph;

/// We'll need slices of several vectors at once.
struct SortingSlices<'a, T: 'a> {
    s: Vec<&'a mut [T]>,
    i: usize,
}

impl<'a, T: 'a + Ord + Sync + Copy + Send> SortingSlices<'a, T> {
    /// Call parallel merge on the right slices.
    fn fuse(self, right: Self, block_size_fuse: usize) -> Self {
        let mut left = self;
        let mut right = right;

        let destination_index = {
            let destination_index = (0..3).find(|&x| x != left.i && x != right.i).unwrap();
            {
                let left_index = left.i;
                let right_index = right.i;

                let (left_input, left_output) = left.mut_couple(left_index, destination_index);
                let (right_input, right_output) = right.mut_couple(right_index, destination_index);

                let output_slice = fuse_multiple_slices!(left_output, right_output);
                #[cfg(not(feature = "logs"))]
                merge_2_par(left_input, right_input, output_slice, block_size_fuse);

                #[cfg(feature = "logs")]
                //subgraph("Fuse rec master", full_size, || {
                merge_2_par(left_input, right_input, output_slice, block_size_fuse);
                //});
            }
            destination_index
        };
        let fused_slices: Vec<_> = left
            .s
            .into_iter()
            .zip(right.s.into_iter())
            .map(|(left_s, right_s)| fuse_multiple_slices!(left_s, right_s))
            .collect();
        SortingSlices {
            s: fused_slices,
            i: destination_index,
        }
    }

    /// Borrow all mutable slices at once.
    fn mut_slices(&mut self) -> (&mut [T], &mut [T], &mut [T]) {
        let (s0, leftover) = self.s.split_first_mut().unwrap();
        let (s1, s2) = leftover.split_first_mut().unwrap();
        (s0, s1, s2[0])
    }
    /// Return the two mutable slices of given indices.
    fn mut_couple(&mut self, i1: usize, i2: usize) -> (&mut [T], &mut [T]) {
        let (s0, s1, s2) = self.mut_slices();
        match (i1, i2) {
            (0, 1) => (s0, s1),
            (0, 2) => (s0, s2),
            (1, 0) => (s1, s0),
            (1, 2) => (s1, s2),
            (2, 0) => (s2, s0),
            (2, 1) => (s2, s1),
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

/// Parallel sort join 2 to 2
pub fn adaptive_sort_join2<T: Ord + Copy + Send + Sync>(
    slice: &mut [T],
    block_size: usize,
    block_size_fuse: usize,
) {
    let mut tmp_slice1 = Vec::with_capacity(slice.base_length().unwrap());
    let mut tmp_slice2 = Vec::with_capacity(slice.base_length().unwrap());
    unsafe {
        tmp_slice1.set_len(slice.base_length().unwrap());
        tmp_slice2.set_len(slice.base_length().unwrap());
    }

    let slices = SortingSlices {
        s: vec![slice, tmp_slice1.as_mut_slice(), tmp_slice2.as_mut_slice()],
        i: 0,
    };

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
    let mut result_slices = schedule_join2(
        k,
        &|l: SortingSlices<T>, r| l.fuse(r, block_size_fuse),
        block_size,
    );

    #[cfg(feature = "logs")]
    let mut result_slices = schedule_join2(
        k,
        &|l: SortingSlices<T>, r| subgraph("Fuse", 2 * l.s[0].len(), || l.fuse(r, block_size_fuse)),
        block_size,
    );

    if result_slices.i != 0 {
        let i = result_slices.i;
        let (destination, source) = result_slices.mut_couple(0, i);
        destination.copy_from_slice(source);
    }
}
