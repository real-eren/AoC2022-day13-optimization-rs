use logos::{Lexer, Logos};
use std::cmp::Ordering;

pub const DESCRIPTION: &str = "lexer generated with `logos`";

pub fn day13(input: &str) -> usize {
    input
        .split("\n\n")
        .map(|chunk| {
            chunk
                .split_once('\n')
                .unwrap_or_else(|| panic!("strange format: {chunk}"))
        })
        .map(|(left, right)| compare(left, right))
        .enumerate()
        .filter_map(|(idx, ord)| if ord.is_lt() { Some(idx + 1) } else { None })
        .sum()
}

fn compare(left: &str, right: &str) -> Ordering {
    fn next_comparable_token(lexer: &mut Lexer<Token>, depth: &mut usize) -> Option<Token> {
        loop {
            match lexer.next()?.unwrap() {
                Token::Comma => panic!("didn't expect comma at index {}", lexer.span().start),
                Token::LBrace => *depth += 1,
                token => return Some(token),
            }
        }
    }
    let mut left = Token::lexer(left);
    let mut left_depth = 0;

    let mut right = Token::lexer(right);
    let mut right_depth = 0;

    loop {
        let left_token = next_comparable_token(&mut left, &mut left_depth);
        let right_token = next_comparable_token(&mut right, &mut right_depth);

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
                            Token::LBrace => panic!("'[' immediately after ']' -> expected comma"),
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
#[logos(skip r"[ \t\n\f]+")]
enum Token {
    #[token(",")]
    Comma,

    #[token("[")]
    LBrace,

    #[token("]")]
    RBrace,

    #[regex("[0-9]+")]
    Number,
}
