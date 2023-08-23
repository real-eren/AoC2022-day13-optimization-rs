//! lexer generated with the `logos` crate.

use crate::shared::day13_framework;
use logos::{Lexer, Logos};
use std::cmp::Ordering;

pub fn day13(input: &str) -> usize {
    day13_framework(input, compare)
}

fn compare(left: &str, right: &str) -> Ordering {
    fn next_comparable_token(lexer: &mut Lexer<Token>) -> Option<(Token, usize)> {
        let mut depth_change = 0;
        loop {
            match lexer.next()?.unwrap() {
                Token::Comma => panic!("didn't expect comma at index {}", lexer.span().start),
                Token::LBraces => depth_change += lexer.span().len(),
                token => return Some((token, depth_change)),
            }
        }
    }
    let mut left = Token::lexer(left);
    let mut left_depth = 0;

    let mut right = Token::lexer(right);
    let mut right_depth = 0;

    loop {
        let left_token = next_comparable_token(&mut left);
        let left_token = match left_token {
            Some((tok, d)) => {
                left_depth += d;
                Some(tok)
            }
            None => None,
        };
        let right_token = match next_comparable_token(&mut right) {
            Some((tok, d)) => {
                right_depth += d;
                Some(tok)
            }
            None => None,
        };

        match (left_token, right_token) {
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
            (Some(Token::Number), Some(Token::Number)) => {
                match left.slice().cmp(right.slice()) {
                    Ordering::Equal => {}
                    cmp => return cmp,
                }
                // if one deeper, the deeper one had better be followed by {diff} ']'s, else it's
                // greater
                if left_depth != right_depth {
                    let diff = left_depth.abs_diff(right_depth);
                    let (deeper_chars, deeper_depth, ret_val) = if left_depth < right_depth {
                        (&mut right, &mut right_depth, Ordering::Less)
                    } else {
                        (&mut left, &mut left_depth, Ordering::Greater)
                    };
                    for _ in 0..diff {
                        let Some(token_result) = deeper_chars.next() else {
                            panic!("line ended before closing all the '['s")
                        };
                        let token = token_result.unwrap();
                        match token {
                            Token::RBrace => {}
                            Token::Comma => return ret_val,
                            Token::LBraces => panic!("'[' immediately after ']' -> expected comma"),
                            Token::Number => panic!(
                                "number {} immediately after ']' -> expected comma",
                                deeper_chars.slice()
                            ),
                        }
                    }
                    *deeper_depth -= diff;
                    // both lines at equal depth, equal so far.
                }

                // first item in both lists were equal. Both lists at equal depth now.
            }
            (Some(Token::Number), Some(Token::RBrace)) => {
                return Ordering::Greater;
            }
            (Some(Token::RBrace), Some(Token::Number)) => {
                return Ordering::Less;
            }
            (Some(Token::RBrace), Some(Token::RBrace)) => {
                match left_depth.cmp(&right_depth) {
                    Ordering::Equal => {}
                    cmp => return cmp,
                }
                left_depth -= 1;
                right_depth -= 1;
            }
            (Some(_), Some(_)) => unreachable!(""),
        }

        assert_eq!(left_depth, right_depth);

        // handle following ',' ']' or None
        loop {
            // only loops when both have RBrace
            match (
                left.next().transpose().unwrap(),
                right.next().transpose().unwrap(),
            ) {
                (Some(Token::RBrace), Some(Token::RBrace)) => {
                    left_depth -= 1;
                    right_depth -= 1;
                    continue;
                }
                (Some(Token::Comma), Some(Token::Comma)) => break, // just skip past them
                (Some(Token::Comma), Some(Token::RBrace)) => return Ordering::Greater,
                (Some(Token::RBrace), Some(Token::Comma)) => return Ordering::Less,
                (Some(_), Some(_)) => {
                    panic!("expected comma or closing bracket, got number / open bracket")
                }
                (None, Some(_)) => panic!("invalid char in left line!"),
                (Some(_), None) => panic!("invalid char in left line!"),
                (None, None) => {
                    if left_depth == 0 && right_depth == 0 {
                        return Ordering::Equal;
                    }
                    panic!("Reached end of line before closing all brackets!");
                }
            }
        }
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ ]+")]
enum Token {
    #[token(",")]
    Comma,

    #[regex("\\[+")]
    LBraces,

    #[token("]")]
    RBrace,

    #[regex("[0-9]+")]
    Number,
}
