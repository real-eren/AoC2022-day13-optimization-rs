pub mod input_handling_baseline;
pub mod logos_lex;
pub mod manual_lex;
pub mod naive;
pub mod naive_slice;
pub mod prefix_comp_then_logos_lex;
mod shared;

pub const SAMPLE: &str = "[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";
#[cfg(test)]
mod tests {
    use crate::{logos_lex, manual_lex, naive, naive_slice, prefix_comp_then_logos_lex, SAMPLE};

    #[test]
    fn naive() {
        assert_eq!(naive::no_pool::day13(SAMPLE), 13)
    }

    #[test]
    fn naive_cached() {
        assert_eq!(naive::pooled::day13(SAMPLE), 13)
    }

    #[test]
    fn naive_slice() {
        assert_eq!(naive_slice::no_pool::day13(SAMPLE), 13)
    }

    #[test]
    fn naive_slice_cached() {
        assert_eq!(naive_slice::pooled::day13(SAMPLE), 13)
    }

    #[test]
    fn manual_lex() {
        assert_eq!(manual_lex::day13(SAMPLE), 13)
    }

    #[test]
    fn logos_lex() {
        assert_eq!(logos_lex::day13(SAMPLE), 13)
    }

    #[test]
    fn skip_prefix_then_lex() {
        assert_eq!(prefix_comp_then_logos_lex::day13::<128>(SAMPLE), 13)
    }
}
