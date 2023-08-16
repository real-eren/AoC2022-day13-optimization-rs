/// Dependency for day13 implementations.
use std::cmp::Ordering;

/// Outline of a solution - extracts pairs and passes them to the given line comparator
#[inline(always)]
pub(crate) fn day13_framework(
    mut input: &str,
    mut line_comparator: impl FnMut(&str, &str) -> Ordering,
) -> usize {
    let mut count = 0;
    let mut idx = 1;
    while !input.is_empty() {
        let Some((left, rem)) = input.split_once('\n') else {break};
        let (right, rem) = rem.split_once('\n').unwrap_or((rem, ""));

        if line_comparator(left, right).is_lt() {
            count += idx;
        }

        input = rem.trim_start_matches('\n');
        idx += 1;
    }
    count
}
