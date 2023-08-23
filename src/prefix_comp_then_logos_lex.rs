//! skip common prefix in inputs, then lex with logos until decision made. *Does not fully validate input*  

use crate::shared::day13_framework;
use logos::{Lexer, Logos};
use std::{cmp::Ordering, iter};

pub fn day13<const N: usize>(input: &str) -> usize {
    day13_framework(input, compare::<N>)
}

fn compare<const N: usize>(left: &str, right: &str) -> Ordering {
    // nkkarpov - https://users.rust-lang.org/t/how-to-find-common-prefix-of-two-byte-slices-effectively/25815/4
    /// returns length of common prefix / index of first mismatch
    fn mismatch<const N: usize>(l: &[u8], r: &[u8]) -> usize {
        mismatch_chunks::<N>(l, r)
    }
    fn mismatch_chunks<const N: usize>(l: &[u8], r: &[u8]) -> usize {
        let off = iter::zip(l.chunks_exact(N), r.chunks_exact(N))
            .take_while(|(l, r)| l == r)
            .count()
            * N;
        off + iter::zip(&l[off..], &r[off..])
            .take_while(|(x, y)| x == y)
            .count()
    }

    fn next_comparable_token(lexer: &mut Lexer<Token>) -> Option<(Token, usize)> {
        let mut depth_change = 0;
        loop {
            match lexer.next()?.unwrap() {
                Token::Comma => panic!("didn't expect comma"),
                Token::LBrace => depth_change += 1,
                token => return Some((token, depth_change)),
            }
        }
    }

    // these will get overwritten with shorter slices throughout the loop
    let mut left = left.as_bytes();
    let mut right = right.as_bytes();

    // skip common prefix, lex until decide or equal, then loop
    loop {
        let common_prefix_length = mismatch::<N>(left, right);
        match (
            common_prefix_length == left.len(),
            common_prefix_length == right.len(),
        ) {
            (false, false) => {}
            (true, true) => return Ordering::Equal,
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
        }

        let (left_char, right_char) = (left[common_prefix_length], right[common_prefix_length]);
        left = &left[common_prefix_length..];
        right = &right[common_prefix_length..];

        #[derive(Clone, Copy)]
        enum WhichIsList {
            Left = -1,
            Right = 1,
        }
        let which_is_list = match (left_char, right_char) {
            (b'0'..=b'9', b'0'..=b'9') => return left_char.cmp(&right_char), // won't return Equal
            // because this is by definition a mismatch point
            (b',' | b'[', b']') | (b'0'..=b'9', b']' | b',') => return Ordering::Greater,
            (b']', b',' | b'[') | (b']' | b',', b'0'..=b'9') => return Ordering::Less,
            (b'[', b'[') | (b']', b']') | (b',', b',') => unreachable!(),
            (b'[', b',') | (b',', b'[') => panic!("invalid syntax"),
            (b'0'..=b'9', b'[') => WhichIsList::Right,
            (b'[', b'0'..=b'9') => WhichIsList::Left,
            (left, right) => panic!("invalid input: at least one of :`{left}`, `{right}`"),
        };
        // at this point, one is pointing to the start of a number, the other to the start of a list
        {
            let (deeper_input, other_input) = match which_is_list {
                WhichIsList::Left => (left, right),
                WhichIsList::Right => (right, left),
            };
            let mut deeper_lexer = Token::lexer(deeper_input);
            let (deeper_first_comparable_token, depth_diff) =
                next_comparable_token(&mut deeper_lexer).unwrap_or_else(|| {
                    panic!("invalid token in right line after position {common_prefix_length}")
                });
            let deeper_number = match deeper_first_comparable_token {
                Token::Comma | Token::LBrace => unreachable!(),
                Token::RBrace => return Ordering::Greater,
                Token::Number => deeper_lexer.slice(),
            };

            let other_number = {
                let mut lexer = Token::lexer(other_input);
                lexer.next();
                match which_is_list {
                    WhichIsList::Left => right = lexer.remainder(),
                    WhichIsList::Right => left = lexer.remainder(),
                }
                lexer.slice()
            };
            match (deeper_number.cmp(other_number), which_is_list) {
                (Ordering::Equal, _) => {}
                (cmp, WhichIsList::Left) => return cmp,
                (cmp, WhichIsList::Right) => return cmp.reverse(),
            }

            for _ in 0..depth_diff {
                match deeper_lexer
                    .next()
                    .transpose()
                    .unwrap()
                    .expect("missing closing brackets")
                {
                    Token::Comma => {
                        return match which_is_list {
                            WhichIsList::Left => Ordering::Less,
                            WhichIsList::Right => Ordering::Greater,
                        }
                    }
                    Token::LBrace => panic!("expected comma before '['"),
                    Token::RBrace => {}
                    Token::Number => panic!("expected comma before number"),
                }
            }
            match which_is_list {
                WhichIsList::Left => left = deeper_lexer.remainder(),
                WhichIsList::Right => right = deeper_lexer.remainder(),
            }
            // both streams are equal now
        }
        // equivalent so far, next is either ',' or ']'
        // match on that ( loop on (], ]) )
        // could do another common_prefix check here, but currently not sure if we'd know what
        // state we're in. Given that we are assuming valid syntax, it may very well be possible,
        // and is worth investigating.
        loop {
            let (left_char, right_char) = match (left.first(), right.first()) {
                (None, None) => return Ordering::Equal,
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some(l), Some(r)) => (*l, *r),
            };
            left = &left[1..];
            right = &right[1..];
            match (left_char, right_char) {
                (b']', b']') => {
                    continue;
                }
                (b',', b',') => break,
                (b']', b',') => return Ordering::Less,
                (b',', b']') => return Ordering::Greater,
                (l, r) => panic!("invalid character in at least one of `{l}`, `{r}`"),
            }
        }
        // now both are looking at the start of the next element, repeat
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token(b",")]
    Comma,

    #[token("[")]
    LBrace,

    #[token("]")]
    RBrace,

    #[regex("[0-9]+")]
    Number,
}
