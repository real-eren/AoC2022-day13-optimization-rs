//! shared input parsing framework w/ trivial compare function

use crate::shared::day13_framework;
use std::cmp::Ordering;

pub fn day13(input: &str) -> usize {
    day13_framework(input, compare)
}
fn compare(left: &str, right: &str) -> Ordering {
    left.len()
        .cmp(&right.len())
        .then(left.as_bytes()[0].cmp(&right.as_bytes()[0]))
}
