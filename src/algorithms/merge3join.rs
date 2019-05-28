//! adaptive parallel merge sort.
// use crate::divisibility::divisible::{lllllDivisible;
// use crate::schedulers::schedule_join3;
// use crate::utils::merge_2;
use crate::prelude::*;
// use rayon_adaptive::Policy;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::prelude::*;

#[cfg(feature = "logs")]
use rayon_logs::subgraph;
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

/// Takes 3 slices and returns the merged data in vector
/// This function is iterative
pub(crate) fn merge_3<'a, T: 'a + Ord + Copy>(s1: &[T], s2: &[T], s3: &[T], mut v: &mut [T]) {
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

/// Cut sorted slice `slice` around start point, splitting around
/// all values equal to value at start point.
/// cost is O(log(|removed part size|))
fn split_around<T: Eq>(slice: &[T], start: usize) -> (&[T], &[T], &[T]) {
    let low_slice = subslice_without_last_value(&slice[0..=start]);
    let high_slice = subslice_without_first_value(&slice[start..]);
    let equal_slice = &slice[low_slice.len()..slice.len() - high_slice.len()];
    (low_slice, equal_slice, high_slice)
}

/// Performs a recursive merge
///  * Cuts the larger array in 3
///  * Binary search the 1/3 and 2/3 values in the other arrays
///  * Recursively merge the sub arrays
fn merge_3_par_aux<'a, T: 'a + Ord + Copy + Sync + Send>(
    large: &'a [T],
    mid: &'a [T],
    small: &'a [T],
) -> (
    (usize, (&'a [T], &'a [T], &'a [T])),
    (usize, (&'a [T], &'a [T], &'a [T])),
    (usize, (&'a [T], &'a [T], &'a [T])),
    (usize, (&'a [T], &'a [T], &'a [T])),
    (usize, (&'a [T], &'a [T], &'a [T])),
) {
    let len_large = large.len();
    let first_third = len_large / 3;

    // ----- FIRST THIRD -----
    let split_large_first_third = split_around(large, first_third);
    let split_mid_first_third = match mid.binary_search(&large[first_third]) {
        Ok(i) => split_around(mid, i),
        Err(i) => {
            let (mid1, mid3) = mid.split_at(i);
            (mid1, &mid[0..0], mid3)
        }
    };
    let split_small_first_third = match small.binary_search(&large[first_third]) {
        Ok(i) => split_around(small, i),
        Err(i) => {
            let (small1, small3) = small.split_at(i);
            (small1, &small[0..0], small3)
        }
    };
    // ----- SECOND THIRD -----
    //
    // TODO: What if this part is empty ?
    let second_third = split_large_first_third.2.len() / 2;

    let split_large_second_third = split_around(split_large_first_third.2, second_third);
    let split_mid_second_third = match split_mid_first_third
        .2
        .binary_search(&split_large_first_third.2[second_third])
    {
        Ok(i) => split_around(split_mid_first_third.2, i),
        Err(i) => {
            let (mid1, mid3) = split_mid_first_third.2.split_at(i);
            (mid1, &split_mid_first_third.2[0..0], mid3)
        }
    };
    let split_small_second_third = match split_small_first_third
        .2
        .binary_search(&split_large_first_third.2[second_third])
    {
        Ok(i) => split_around(split_small_first_third.2, i),
        Err(i) => {
            let (small1, small3) = split_small_first_third.2.split_at(i);
            (small1, &split_small_first_third.2[0..0], small3)
        }
    };
    // [ less than pivot 1 | pivot(s) 1 | more p1, less p2 | pivot(s) 2 | more p2 ]
    // ^                   ^            ^                  ^            ^
    // |                   |            |                  |            |
    // 0                   i1           i2                 i3           i4

    let i1 = split_large_first_third.0.len()
        + split_mid_first_third.0.len()
        + split_small_first_third.0.len();
    let i2 = i1
        + split_large_first_third.1.len()
        + split_mid_first_third.1.len()
        + split_small_first_third.1.len();
    let i3 = i2
        + split_large_second_third.0.len()
        + split_mid_second_third.0.len()
        + split_small_second_third.0.len();
    let i4 = i3
        + split_large_second_third.1.len()
        + split_mid_second_third.1.len()
        + split_small_second_third.1.len();

    return (
        (
            0,
            (
                &split_large_first_third.0,
                &split_mid_first_third.0,
                &split_small_first_third.0,
            ),
        ),
        (
            i1,
            (
                &split_large_first_third.1,
                &split_mid_first_third.1,
                &split_small_first_third.1,
            ),
        ),
        (
            i2,
            (
                &split_large_second_third.0,
                &split_mid_second_third.0,
                &split_small_second_third.0,
            ),
        ),
        (
            i3,
            (
                &split_large_second_third.1,
                &split_mid_second_third.1,
                &split_small_second_third.1,
            ),
        ),
        (
            i4,
            (
                &split_large_second_third.2,
                &split_mid_second_third.2,
                &split_small_second_third.2,
            ),
        ),
    );
}

/// Parallel merge 3 to 3
fn merge_3_par<'a, T: 'a + Ord + Copy + Sync + Send>(
    s1: &[T],
    s2: &[T],
    s3: &[T],
    mut v: &mut [T],
    min_size: usize,
) {
    let len1 = s1.len();
    let len2 = s2.len();
    let len3 = s3.len();

    if len1 <= min_size || len2 <= min_size || len3 <= min_size {
        // merge_3(s1, s2, s3, &mut v);
        subgraph("Merge seq", min_size, || merge_3(s1, s2, s3, &mut v));
    } else {
        let (
            (_, (ft1, ft2, ft3)),
            (i1, (pv1_1, pv1_2, pv1_3)),
            (i2, (st1, st2, st3)),
            (i3, (pv2_1, pv2_2, pv2_3)),
            (i4, (tt1, tt2, tt3)),
        ) = if len1 >= len2 && len1 >= len3 {
            merge_3_par_aux(&s1, &s2, &s3)
        } else if len2 >= len1 && len2 >= len3 {
            let (
                (i0, (ft1, ft2, ft3)),
                (i1, (pv1_1, pv1_2, pv1_3)),
                (i2, (st1, st2, st3)),
                (i3, (pv2_1, pv2_2, pv2_3)),
                (i4, (tt1, tt2, tt3)),
            ) = merge_3_par_aux(&s2, &s1, &s3);
            (
                (i0, (ft2, ft1, ft3)),
                (i1, (pv1_2, pv1_1, pv1_3)),
                (i2, (st2, st1, st3)),
                (i3, (pv2_2, pv2_1, pv2_3)),
                (i4, (tt2, tt1, tt3)),
            )
        } else {
            let (
                (i0, (ft1, ft2, ft3)),
                (i1, (pv1_1, pv1_2, pv1_3)),
                (i2, (st1, st2, st3)),
                (i3, (pv2_1, pv2_2, pv2_3)),
                (i4, (tt1, tt2, tt3)),
            ) = merge_3_par_aux(&s3, &s2, &s1);
            (
                (i0, (ft3, ft2, ft1)),
                (i1, (pv1_3, pv1_2, pv1_1)),
                (i2, (st3, st2, st1)),
                (i3, (pv2_3, pv2_2, pv2_1)),
                (i4, (tt3, tt2, tt1)),
            )
        };
        let x = s1[0];
        let mut v_first_third = vec![x; i1];

        let mut v_second_third = vec![x; i3 - i2];

        subgraph("Fuse par", len1 + len2 + len3, || {
            rayon::join(
                || {
                    rayon::join(
                        || merge_3_par(&ft1, &ft2, &ft3, &mut v_first_third, min_size),
                        || merge_3_par(&st1, &st2, &st3, &mut v_second_third, min_size),
                    )
                },
                || merge_3_par(&tt1, &tt2, &tt3, &mut v[i4..], min_size),
            );
        });

        v[..i1].copy_from_slice(&v_first_third);
        v[i1..i1 + pv1_1.len()].copy_from_slice(&pv1_1);
        v[i1 + pv1_1.len()..i1 + pv1_1.len() + pv1_2.len()].copy_from_slice(&pv1_2);
        v[i1 + pv1_1.len() + pv1_2.len()..i1 + pv1_1.len() + pv1_2.len() + pv1_3.len()]
            .copy_from_slice(&pv1_3);
        v[i2..i3].copy_from_slice(&v_second_third);
        v[i3..i3 + pv2_1.len()].copy_from_slice(&pv2_1);
        v[i3 + pv2_1.len()..i3 + pv2_1.len() + pv2_2.len()].copy_from_slice(&pv2_2);
        v[i3 + pv2_1.len() + pv2_2.len()..i3 + pv2_1.len() + pv2_2.len() + pv2_3.len()]
            .copy_from_slice(&pv2_3);
    }
}

// sort related code

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

/// Parallel sort join 3 to 3
pub fn adaptive_sort_join3<T: Ord + Copy + Send + Sync>(
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
    let mut result_slices = schedule_join3(
        k,
        &|l: SortingSlices<T>, m, r| l.fuse(m, r, block_size_fuse),
        block_size,
    );

    #[cfg(feature = "logs")]
    let mut result_slices = schedule_join3(
        k,
        &|l: SortingSlices<T>, m, r| {
            subgraph("Fuse", 3 * l.s[0].len(), || l.fuse(m, r, block_size_fuse))
        },
        block_size,
    );

    if result_slices.i != 0 {
        let i = result_slices.i;
        let (destination, source) = result_slices.mut_couple(0, i);
        destination.copy_from_slice(source);
    }
}
