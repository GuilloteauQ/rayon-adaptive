//! adaptive parallel merge sort.
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use std::iter::repeat;

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
// main related code

/// find subslice without last value in given sorted slice.
fn subslice_without_last_value<T: Eq>(slice: &[T]) -> &[T] {
    match slice.split_last() {
        Some((target, slice)) => {
            let searching_range_start = repeat(())
                .scan(1, |acc, _| {
                    *acc *= 2;
                    Some(*acc)
                }) // iterate on all powers of 2
                .take_while(|&i| i < slice.len())
                .map(|i| slice.len() - i) // go farther and farther from end of slice
                .find(|&i| unsafe { slice.get_unchecked(i) != target })
                .unwrap_or(0);

            let index = slice[searching_range_start..]
                .binary_search_by(|x| {
                    if x.eq(target) {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Less
                    }
                })
                .unwrap_err();
            &slice[0..(searching_range_start + index)]
        }
        None => slice,
    }
}

/// find subslice without first value in given sorted slice.
fn subslice_without_first_value<T: Eq>(slice: &[T]) -> &[T] {
    match slice.first() {
        Some(target) => {
            let searching_range_end = repeat(())
                .scan(1, |acc, _| {
                    *acc *= 2;
                    Some(*acc)
                }) // iterate on all powers of 2
                .take_while(|&i| i < slice.len())
                .find(|&i| unsafe { slice.get_unchecked(i) != target })
                .unwrap_or_else(|| slice.len());

            let index = slice[..searching_range_end]
                .binary_search_by(|x| {
                    if x.eq(target) {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                })
                .unwrap_err();
            &slice[index..]
        }
        None => slice,
    }
}

/// Cut sorted slice `slice` around start point, splitting around
/// all values equal to value at start point.
/// cost is O(log(|removed part size|))
fn split_around<T: Eq>(slice: &[T], start: usize) -> (&[T], &[T], &[T]) {
    let low_slice = subslice_without_last_value(&slice[0..=start]);
    let high_slice = subslice_without_first_value(&slice[start..]);
    let equal_slice = &slice[low_slice.len()..slice.len() - high_slice.len()];
    (low_slice, equal_slice, high_slice)
}

/// split large array at midpoint and small array where needed for merge.
fn merge_split<'a, T: Ord>(
    large: &'a [T],
    small: &'a [T],
) -> ((&'a [T], &'a [T], &'a [T]), (&'a [T], &'a [T], &'a [T])) {
    let middle = large.len() / 2;
    let split_large = split_around(large, middle);
    let split_small = match small.binary_search(&large[middle]) {
        Ok(i) => split_around(small, i),
        Err(i) => {
            let (small1, small3) = small.split_at(i);
            (small1, &small[0..0], small3)
        }
    };
    (split_large, split_small)
}

/// Takes 3 slices and returns the merged data in vector
/// This function is iterative
pub(crate) fn merge_3<'a, T: 'a + Ord + Copy>(s1: &[T], s2: &[T], s3: &[T], v: &mut [T]) {
    let len1 = s1.len();
    let len2 = s2.len();
    let len3 = s3.len();
    let mut index1 = 0;
    let mut index2 = 0;
    let mut index3 = 0;
    let mut index_output = 0;
    while index1 < len1 && index2 < len2 && index3 < len3 {
        if s1[index1] <= s2[index2] && s1[index1] <= s3[index3] {
            v[index_output] = s1[index1];
            index1 += 1;
        } else if s2[index2] <= s1[index1] && s2[index2] <= s3[index3] {
            v[index_output] = s2[index2];
            index2 += 1;
        } else {
            v[index_output] = s3[index3];
            index3 += 1;
        }
        index_output += 1;
    }
    if index1 == len1 {
        merge_2(s2, index2, s3, index3, &mut v, index_output);
    } else if index2 == len2 {
        merge_2(s1, index1, s3, index3, &mut v, index_output);
    } else {
        merge_2(s1, index1, s2, index2, &mut v, index_output);
    }
}

// sort related code

/// We'll need slices of several vectors at once.
struct SortingSlices<'a, T: 'a> {
    s: Vec<&'a mut [T]>,
    i: usize,
}

impl<'a, T: 'a + Ord + Sync + Copy + Send> SortingSlices<'a, T> {
    fn new() -> Self {
        let size = 0;
        let mut tmp_slice1: Vec<T> = Vec::new();
        let mut tmp_slice2: Vec<T> = Vec::new();
        let mut tmp_slice3: Vec<T> = Vec::new();
        unsafe {
            tmp_slice1.set_len(size);
            tmp_slice2.set_len(size);
            tmp_slice3.set_len(size);
        }

        let slices = SortingSlices {
            s: vec![tmp_slice1, tmp_slice2, tmp_slice3],
            i: 0,
        };
        slices
    }

    /// Call parallel merge on the right slices.
    fn fuse(self, mid: Self, right: Self) -> Self {
        let mut left = self;
        let mut mid = mid;
        let mut right = right;
        let destination_index = {
            let destination_index = (0..3)
                .find(|&x| x != left.i && x != mid.i && x != right.i)
                .unwrap();
            {
                let left_index = left.i;
                let mid_index = mid.i;
                let right_index = right.i;

                let (left_input, left_output) = left.mut_couple(left_index, destination_index);
                let (mid_input, mid_output) = mid.mut_couple(mid_index, destination_index);
                let (right_input, right_output) = right.mut_couple(right_index, destination_index);

                let output_slice = fuse_multiple_slices!(left_output, mid_output, right_output);
                merge_3(left_input, mid_input, right_input, output_slice);
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

pub fn adaptive_sort_raw_with_policies<T: Ord + Copy + Send + Sync>(slice: &mut [T]) {
    let mut tmp_slice1 = Vec::with_capacity(slice.base_length().unwrap());
    let mut tmp_slice2 = Vec::with_capacity(slice.base_length().unwrap());
    unsafe {
        tmp_slice1.set_len(slice.base_length().unwrap());
        tmp_slice2.set_len(slice.base_length().unwrap());
    }

    let slice_len = slice.len();
    let num_threads = rayon::current_num_threads();

    let slices = SortingSlices {
        s: vec![slice, tmp_slice1.as_mut_slice(), tmp_slice2.as_mut_slice()],
        i: 0,
    };

    let k = slices.work(|mut slices, size| {
        slices.s[slices.i][0..size].sort();
        slices
    });

    let v_final = schedule_join3(
        k,
        &SortingSlices::new, // MARAME WALLAH CEST QUOI IDENTITY
        &|l: SortingSlices<T>, m, r| l.fuse(m, r),
        10,
    );

    // let mut result_slices = slices.with_policy(sort_policy).map_reduce(
    //     |mut slices| {
    //         slices.s[slices.i].sort();
    //         slices
    //     },
    //     |s1, s2, s3| s1.fuse_with_policy(s2, s3),
    // );

    // if result_slices.i != 0 {
    //     let i = result_slices.i;
    //     let (destination, source) = result_slices.mut_couple(0, i);
    //     destination.copy_from_slice(source);
    // }
}
fn main() {
    let v: Vec<u32> = (0..100_000).collect();
    let mut inverted_v: Vec<u32> = (0..100_000).rev().collect();
    adaptive_sort_raw_with_policies(&mut inverted_v);
    assert_eq!(v, inverted_v);
}
