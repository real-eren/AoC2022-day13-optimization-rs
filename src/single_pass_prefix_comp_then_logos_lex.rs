//! like [crate::prefix_comp_then_logos_lex], but lazily finds the right line.

use logos::{Lexer, Logos};
use std::{cmp::Ordering, iter};

pub fn day13(mut input: &str) -> usize {
    let mut count = 0;
    let mut idx = 1;
    while !input.is_empty() {
        let Some((left, rem)) = input.split_once('\n') else { break; };

        let (cmp, rem_idx_after_comparison) = compare_first_line(left, rem);
        debug_assert!({
            let max_idx_in_rem = rem.find('\n').unwrap_or(rem.len());
            rem_idx_after_comparison <= max_idx_in_rem
        });

        if cmp.is_lt() {
            count += idx;
        }
        // advance past the end of the right line, up to the start of the next pair if any
        input = rem[rem_idx_after_comparison..]
            .split_once('\n')
            // may or may not have consumed newline during compare subroutine, so stripping '\n'*
            // should be a safe workaround
            .map_or("", |(_, r)| r.trim_start_matches('\n'));
        idx += 1;
    }
    count
}
/// Compares left against the first line in rem.
/// returns (left.cmp(rem.first_line), num_bytes_consumed_from_rem)
fn compare_first_line(left: &str, rem: &str) -> (Ordering, usize) {
    fn next_comparable_token(lexer: &mut Lexer<Token>) -> Option<(Token, usize)> {
        let mut depth_change = 0;
        loop {
            match lexer.next()?.unwrap() {
                Token::Comma => panic!("didn't expect comma"),
                Token::Newline => panic!("missing closing brace for right line"),
                Token::LBrace => depth_change += 1,
                token => return Some((token, depth_change)),
            }
        }
    }
    // find mismatch of left and rem
    // we know left doesn't have a newline, so the mismatch will not occur past the end of the
    // the right line.
    let (mut left_bytes, mut rem_bytes) = (left.as_bytes(), rem.as_bytes());
    let mut index_into_rem = 0;
    // skip common prefix, lex until decide or equal, then loop
    loop {
        let idx_of_first_diff = mismatch::<16>(left_bytes, rem_bytes);
        index_into_rem += idx_of_first_diff;
        // could be equal to length of both - equal
        // could be equal to length of one - that's lesser
        // could be less than either - need to match on byte then possibly do lexing
        match (
            idx_of_first_diff == left_bytes.len(),
            idx_of_first_diff == rem_bytes.len(),
        ) {
            (true, true) => return (Ordering::Equal, left.len()),
            (true, false) => {
                if rem_bytes[idx_of_first_diff] == b'\n' {
                    // left ran out, but rem's next character was newline, so left == right.
                    return (Ordering::Equal, index_into_rem);
                } else {
                    // assuming valid syntax up to this point, last elem must have been number,
                    // and right's number was just longer.
                    return (Ordering::Less, index_into_rem);
                }
            }
            (false, true) => {
                // assuming valid syntax up to this point, left was longer.
                return (Ordering::Greater, index_into_rem);
            }
            (false, false) => { /* need to compare below */ }
        }

        left_bytes = &left_bytes[idx_of_first_diff..];
        rem_bytes = &rem_bytes[idx_of_first_diff..];

        let left_char = left_bytes[0];
        let right_char = rem_bytes[0];

        #[derive(Clone, Copy)]
        enum WhichIsList {
            Left = -1,
            Right = 1,
        }
        let which_is_list = match (left_char, right_char) {
            (b'0'..=b'9', b'0'..=b'9') => return (left_char.cmp(&right_char), index_into_rem), // won't return Equal
            // because this is by definition a mismatch point
            (b',' | b'[', b']') | (b'0'..=b'9', b']' | b',') => {
                return (Ordering::Greater, index_into_rem)
            }
            (b']', b',' | b'[') | (b']' | b',', b'0'..=b'9') => {
                return (Ordering::Less, index_into_rem)
            }
            (b'[', b'[') | (b']', b']') | (b',', b',') | (_, b'\n') => unreachable!(),
            (b'[', b',') | (b',', b'[') => panic!("invalid syntax"),
            (b'0'..=b'9', b'[') => WhichIsList::Right,
            (b'[', b'0'..=b'9') => WhichIsList::Left,
            (left, right) => panic!(
                "invalid input: at least one of :`{}`, `{}`",
                left as char, right as char
            ),
        };
        // at this point, one is pointing to the start of a number, the other to the start of a list
        {
            /*
             * in this block: need to be careful to increment / add to index_into_rem WHEN:
             * 1) returning
             * 2) advancing rem_bytes, but not multiple times per iteration.
             * Note that at each iteration, left_bytes and rem_bytes is re-sliced (prefix gets
             * chopped off), + lexer is created each iteration. So lexer.span().end is the
             * appropriate value.
             */
            let (deeper_input, other_input) = match which_is_list {
                WhichIsList::Left => (left_bytes, rem_bytes),
                WhichIsList::Right => (rem_bytes, left_bytes),
            };
            let mut deeper_lexer = Token::lexer(deeper_input);
            let (deeper_first_comparable_token, depth_diff) =
                next_comparable_token(&mut deeper_lexer).unwrap_or_else(|| {
                    panic!("invalid token in right line after position {idx_of_first_diff}")
                });
            let deeper_number = match deeper_first_comparable_token {
                Token::Comma | Token::Newline | Token::LBrace => unreachable!(),
                Token::RBrace => {
                    return (Ordering::Greater, index_into_rem + deeper_lexer.span().end)
                }
                Token::Number => deeper_lexer.slice(),
            };

            let other_number = {
                let mut lexer = Token::lexer(other_input);
                lexer.next();
                match which_is_list {
                    WhichIsList::Left => {
                        // rem contains just the number - it won't be advanced for the rest of this
                        // block
                        index_into_rem += lexer.span().end;
                        rem_bytes = lexer.remainder();
                    }
                    WhichIsList::Right => left_bytes = lexer.remainder(),
                }
                lexer.slice()
            };
            match (deeper_number.cmp(other_number), which_is_list) {
                (Ordering::Equal, _) => {}
                (cmp, WhichIsList::Left) => return (cmp, index_into_rem),
                (cmp, WhichIsList::Right) => {
                    return (cmp.reverse(), index_into_rem + deeper_lexer.span().end)
                }
            }

            for _ in 0..depth_diff {
                match deeper_lexer
                    .next()
                    .transpose()
                    .unwrap()
                    .expect("missing closing brackets")
                {
                    Token::Comma => {
                        let cmp = match which_is_list {
                            WhichIsList::Left => Ordering::Less,
                            WhichIsList::Right => {
                                index_into_rem += deeper_lexer.span().end;
                                Ordering::Greater
                            }
                        };
                        return (cmp, index_into_rem);
                    }
                    Token::LBrace => panic!("expected comma before '['"),
                    Token::Newline => panic!("missing closing brackets"),
                    Token::RBrace => {}
                    Token::Number => panic!("expected comma before number"),
                }
            }
            match which_is_list {
                WhichIsList::Left => left_bytes = deeper_lexer.remainder(),
                WhichIsList::Right => {
                    index_into_rem += deeper_lexer.span().end;
                    rem_bytes = deeper_lexer.remainder();
                }
            }
            // both streams are equal now
        }
        // equivalent so far, next is either ',' or ']'
        // match on that ( loop on (], ]) )
        loop {
            let (left_char, right_char) = match (left_bytes.first(), rem_bytes.first()) {
                (None, None) => return (Ordering::Equal, index_into_rem),
                (None, Some(_)) => return (Ordering::Less, index_into_rem),
                (Some(_), None) => return (Ordering::Greater, index_into_rem),
                (Some(l), Some(r)) => (*l, *r),
            };
            left_bytes = &left_bytes[1..];
            rem_bytes = &rem_bytes[1..];
            match (left_char, right_char) {
                (b']', b']') => {
                    continue;
                }
                // right can be new line only if invalid syntax
                (_, b'\n') => panic!("right line ended before closing all the brackets"),
                (b',', b',') => break,
                (b']', b',') => return (Ordering::Less, index_into_rem),
                (b',', b']') => return (Ordering::Greater, index_into_rem),
                (l, r) => panic!("invalid character in at least one of `{l}`, `{r}`"),
            }
        }
        // now both are looking at the start of the next element, repeat
    }
}
fn mismatch<const N: usize>(left: &[u8], right: &[u8]) -> usize {
    let off = iter::zip(left.chunks_exact(N), right.chunks_exact(N))
        .take_while(|(l, r)| l == r)
        .count()
        * N;
    off + iter::zip(&left[off..], &right[off..])
        .take_while(|(l, r)| l == r)
        .count()
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ ]+")]
enum Token {
    #[token(b",")]
    Comma,

    #[token("[")]
    LBrace,

    #[token("]")]
    RBrace,

    #[regex("[0-9]+")]
    Number,

    #[token("\n")]
    Newline,
}
