//! misc utilities.
use std::iter::{repeat, successors};

/// Iterator on min_size, min_size*2, min_size*4, ..., max_size, max_size, max_size...
pub(crate) fn power_sizes(min_size: usize, max_size: usize) -> impl Iterator<Item = usize> {
    successors(Some(min_size), |&p| Some(2 * p))
        .take_while(move |&p| p < max_size)
        .chain(repeat(max_size))
}

// use itertools::kmerge;
//
// /// Takes 3 slices and returns the merged data in a vector
// /// Uses Kmerge form Itertools that uses a heap
// pub fn kmerge_3<'a, T: 'a + Ord + Copy>(s1: &[T], s2: &[T], s3: &[T]) -> Vec<T> {
//     kmerge(vec![s1, s2, s3]).cloned().collect()
// }
//
// pub fn kmerge_3_sortie<'a, T: 'a + Ord + Copy>(s1: &[T], s2: &[T], s3: &[T], out: &mut [T]) {
//     for (i, o) in kmerge(vec![s1, s2, s3]).zip(out.iter_mut()) {
//         *o = *i
//     }
// }

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

/// Takes 3 slices and returns the merged data in vector
/// This function is iterative
pub fn merge_3<'a, T: 'a + Ord + Copy>(s1: &[T], s2: &[T], s3: &[T]) -> Vec<T> {
    let len1 = s1.len();
    let len2 = s2.len();
    let len3 = s3.len();
    let mut v = vec![s1[0]; len1 + len2 + len3];
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
    v
}
