//! O(1) space, char-by-char hand-rolled lexer

use crate::shared::day13_framework;
use std::{
    cmp::Ordering,
    iter::{Enumerate, Peekable},
    str::Bytes,
};

pub fn day13(input: &str) -> usize {
    day13_framework(input, compare)
}

fn compare(left: &str, right: &str) -> Ordering {
    enum Token<'a> {
        LBrace,
        RBrace,
        Comma,
        Number(&'a str),
    }

    #[inline(always)]
    fn skip_whitespace(chars: &mut Peekable<Enumerate<Bytes>>) {
        let Some((_, b)) = chars.peek() else { return };
        if *b != b' ' {
            return;
        }
        chars.next();
        while chars
            .next_if(|(_, char)| char.is_ascii_whitespace())
            .is_some()
        {}
    }

    fn next_token<'a>(
        chars: &mut Peekable<Enumerate<Bytes>>,
        source: &'a str,
    ) -> Option<Token<'a>> {
        if let Some((_, b)) = chars.peek() {
            if *b == b' ' {
                skip_whitespace(chars);
            }
        }

        Some(match chars.next()? {
            (_, b'[') => Token::LBrace,
            (_, b']') => Token::RBrace,
            (_, b',') => Token::Comma,
            (start, b'0'..=b'9') => {
                // advance while still numbers
                let mut end = start; // inclusive!
                while let Some((idx, _)) = chars.next_if(|(_, char)| char.is_ascii_digit()) {
                    end = idx;
                }
                Token::Number(source.split_at(end + 1).0.split_at(start).1)
            }
            (idx, char) => panic!("unexpected char {char} at position {idx}"),
        })
    }

    fn next_comparable_token<'a>(
        chars: &mut Peekable<Enumerate<Bytes>>,
        source: &'a str,
        depth: &mut usize,
    ) -> Option<Token<'a>> {
        loop {
            skip_whitespace(chars);
            match chars.peek()? {
                (_, b']' | b'0'..=b'9') => return next_token(chars, source),
                (_, b'[') => {
                    chars.next();
                    *depth += 1;
                }
                (idx, b',') => panic!("didn't expect comma at index {idx}"),
                (idx, char) => panic!("invalid character `{char}` at index `{idx}`"),
            }
        }
    }

    let mut left_chars = left.bytes().enumerate().peekable();
    let mut left_depth = 0;

    let mut right_chars = right.bytes().enumerate().peekable();
    let mut right_depth = 0;

    // loop and compare tokens.
    loop {
        // strategy: advance both past '['s until number (or end of empty list), tracking depth
        let left_token = next_comparable_token(&mut left_chars, left, &mut left_depth);
        let right_token = next_comparable_token(&mut right_chars, right, &mut right_depth);

        // ** if different depths, only the next comparison matters
        match (left_token, right_token) {
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
            (Some(Token::Number(left_num)), Some(Token::Number(right_num))) => {
                match left_num.cmp(right_num) {
                    Ordering::Equal => {}
                    cmp => return cmp,
                }
                // if one deeper, the deeper one had better be followed by {diff} ']'s, else it's
                // greater
                if left_depth != right_depth {
                    let diff = left_depth.abs_diff(right_depth);
                    let (deeper_chars, deeper_depth, deeper_source, ret_val) =
                        if left_depth < right_depth {
                            (&mut right_chars, &mut right_depth, right, Ordering::Less)
                        } else {
                            (&mut left_chars, &mut left_depth, left, Ordering::Greater)
                        };
                    for _ in 0..diff {
                        skip_whitespace(deeper_chars);
                        let current_pos_char = deeper_chars.peek().copied();
                        match next_token(deeper_chars, deeper_source) {
                            Some(Token::RBrace) => {}
                            Some(Token::Comma) => return ret_val,
                            None => panic!("line ended before closing all the '['s"),
                            _ => panic!(
                                "expected comma or ']' at index {}, but got `{}`",
                                current_pos_char.unwrap().0,
                                current_pos_char.unwrap().1,
                            ),
                        }
                    }
                    *deeper_depth -= diff;
                    // both lines at equal depth, equal so far.
                }

                // first item in both lists were equal. Both lists at equal depth now.
            }
            (Some(Token::Number(_)), Some(Token::RBrace)) => {
                return Ordering::Greater;
            }
            (Some(Token::RBrace), Some(Token::Number(_))) => {
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
            match (
                next_token(&mut left_chars, left),
                next_token(&mut right_chars, right),
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
                    panic!("expected comma or closing bracket, got something else")
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
