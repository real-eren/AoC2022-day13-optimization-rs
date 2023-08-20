pub mod input_handling_baseline;
pub mod logos_lex;
pub mod manual_lex;
pub mod naive;
pub mod naive_slice;
pub mod prefix_comp_then_logos_lex;
mod shared;
pub mod single_pass_prefix_comp_then_logos_lex;

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

const OTHER: &str = "[51246543,3456543,[[23456]]]
[51246543,3456543,23476]

[4565,[9298,[[266],756],54234],9876,13513]
[4565,[9298,[266],890],2345,13513]

[4565,[9298,[[266],756],54234],9876,13513,91]
[4565,[9298,[266,756],54234],9876,13513,91]

[4565,[9298,[[266],756],54234],9876,13513]
[4565,[9298,[266],890],2345,13513]

[[[[[6531]]]],[[4123]],1,[123]]
[[6532],4123,[[[[1]]]],[123]]

[1565,[9298,[[266],756],54234],9876,13513]
[1565,[9298,[266],890],2345,13513]

[4565,[9298,[[266],756],54234],9876,13513]
[4565,[9298,[266,[[[756]]]],890],2345,13513]

[1,[2,[3,[4,[5,[6,[7]]]]]]]
[1,[2,[3,[4,[5,[6,[7]]]]]],8]

[[[[[[[[5],4],3],2],1],0],1]]
[[[[[[[5],4],3],2],1],0]]

[[987654321],[123456789],[[987654321]],[[123456789]],987654320]
[987654321,123456789,987654321,123456789,[[987654321]]]";
// */
#[cfg(test)]
mod tests {
    use duplicate::duplicate;

    use crate::{
        logos_lex, manual_lex, naive, naive_slice, prefix_comp_then_logos_lex,
        single_pass_prefix_comp_then_logos_lex, OTHER, SAMPLE,
    };

    duplicate! {
        [
            func name;
            [naive::pooled::day13] [naive_pool];
            [naive::no_pool::day13] [naive_no_pool];
            [naive_slice::no_pool::day13] [naive_slice_no_pool];
            [naive_slice::pooled::day13] [naive_slice_pool];
            [manual_lex::day13] [manual_lex_pool];
            [logos_lex::day13] [logos_lex];
            [prefix_comp_then_logos_lex::day13::<16>] [prefix_comp_then_logos_lex];
            [single_pass_prefix_comp_then_logos_lex::day13] [single_pass_prefix_comp];
        ]
        #[test]
        fn name() {
            assert_eq!(func(SAMPLE), 13);
            assert_eq!(func(OTHER), 1 + 0 + 5 + 7 + 8 + 10);
            assert_eq!(func("[]\n[]"), 0);
        }
    }
}
