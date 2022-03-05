//! Sorts for small arrays using sorting networks from: https://bertdobbelaere.github.io/sorting_networks.html
use std::cmp::Ordering;

#[inline(always)]
fn compare_swap<T, F>(arr: &mut [T], cmp: F, lhs: usize, rhs: usize)
where
    F: FnOnce(&T, &T) -> Option<Ordering>,
{
    if Some(Ordering::Greater) == cmp(&arr[lhs], &arr[rhs]) {
        arr.swap(lhs, rhs)
    }
}

/// A layer of comparisons in the sorting network that can be executed in paralell.
/// This is mostly here for clarity.
macro_rules! layer {
    ($arr:ident, $cmp:expr, $(($lhs:expr, $rhs:expr)),+ $(,)?) => {
        $( compare_swap($arr, &$cmp, $lhs, $rhs); )*
    };
}

#[inline]
pub fn sort4_with<T, F>(arr: &mut [T], cmp: F)
where
    F: Fn(&T, &T) -> Option<Ordering>,
{
    layer!(arr, cmp, (0, 2), (1, 3));
    layer!(arr, cmp, (0, 1), (2, 3));
    layer!(arr, cmp, (1, 2));
}

#[inline]
pub fn sort5_with<T, F>(arr: &mut [T], cmp: F)
where
    F: Fn(&T, &T) -> Option<Ordering>,
{
    layer!(arr, cmp, (0, 3), (1, 4));
    layer!(arr, cmp, (0, 2), (1, 3));
    layer!(arr, cmp, (0, 1), (2, 4));
    layer!(arr, cmp, (1, 2), (3, 4));
    layer!(arr, cmp, (2, 3));
}
