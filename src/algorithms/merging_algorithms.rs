use crate::prelude::*;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

#[cfg(feature = "logs")]
use rayon_logs::subgraph;
use std::iter::repeat;

#[macro_export]
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

/// Takes 2 slices as well as their staring index
/// and fill 'r' with the merged data (also update o)
pub fn merge_2<'a, T: 'a + Ord + Copy>(
    s1: &[T],
    mut i1: usize,
    s2: &[T],
    mut i2: usize,
    r: &mut [T],
    mut o: usize,
) {
    let len1 = s1.len();
    let len2 = s2.len();

    // if len1 == 0 {
    //     r[o..].copy_from_slice(&s2[i2..]);
    //     return;
    // }
    // if len2 == 0 {
    //     r[o..].copy_from_slice(&s1[i1..]);
    //     return;
    // }

    // if s1.last() < s2.first() {
    //     r[o..].copy_from_slice(&s1[i1..]);
    //     r[o + len1 - i1..].copy_from_slice(&s2[i2..]);
    //     return;
    // }
    // if s1.first() > s2.last() {
    //     r[o..].copy_from_slice(&s2[i2..]);
    //     r[o + len2 - i2..].copy_from_slice(&s1[i1..]);
    //     return;
    // }
    while i1 < len1 && i2 < len2 {
        if s1[i1] <= s2[i2] {
            r[o] = s1[i1];
            i1 += 1;
        } else {
            r[o] = s2[i2];
            i2 += 1;
        }
        o += 1;
    }
    if i1 < len1 {
        r[o..].copy_from_slice(&s1[i1..]);
    } else {
        r[o..].copy_from_slice(&s2[i2..]);
    }
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
pub(crate) fn merge_3_par<'a, T: 'a + Ord + Copy + Sync + Send>(
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
