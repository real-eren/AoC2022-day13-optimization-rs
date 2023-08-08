use std::cmp::Ordering;

/// Outline of a solution - extracts pairs and passes them to the given line comparator
pub(crate) fn day13_framework(
    input: &str,
    mut line_comparator: impl FnMut(&str, &str) -> Ordering,
) -> usize {
    input
        .split("\n\n")
        .map(|chunk| {
            chunk
                .split_once('\n')
                .unwrap_or_else(|| panic!("strange format: {chunk}"))
        })
        .map(|(l, r)| line_comparator(l, r))
        .enumerate()
        .filter_map(|(idx, ord)| if ord.is_lt() { Some(idx + 1) } else { None })
        .sum()
}