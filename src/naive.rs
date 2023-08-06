use std::{cmp::Ordering, iter::Peekable};

pub const DESCRIPTION: &str = "Parses each line into Vec< Vec | String>";

pub fn day13(input: &str) -> usize {
    input
        .split("\n\n")
        .map(|chunk| {
            chunk
                .split_once('\n')
                .unwrap_or_else(|| panic!("strange format: {chunk}"))
        })
        .map(|(left, right)| Element::parse(left).cmp(&Element::parse(right)))
        .enumerate()
        .filter_map(|(idx, ord)| if ord.is_lt() { Some(idx + 1) } else { None })
        .sum()
}

#[derive(PartialEq, Eq, Debug)]
enum Element {
    Num(String),
    List(Vec<Element>),
}

impl PartialOrd for Element {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Element {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Element::Num(s), Element::Num(o)) => s.cmp(o),
            (Element::Num(_), Element::List(o)) => {
                if o.is_empty() {
                    return Ordering::Greater;
                }
                let first_elem_cmp = self.cmp(&o[0]);
                if first_elem_cmp.is_eq() && o.len() > 1 {
                    Ordering::Less
                } else {
                    first_elem_cmp
                }
            }
            (Element::List(_), Element::Num(_)) => other.cmp(self).reverse(),
            (Element::List(s), Element::List(o)) => s.cmp(o),
        }
    }
}

impl Element {
    fn parse(s: &str) -> Element {
        let s = s.trim();
        if s.chars().all(|ch| ch.is_ascii_digit()) {
            return Self::Num(s.to_owned());
        }

        fn parse_number(chars: &mut Peekable<impl Iterator<Item = char>>) -> String {
            let mut s = String::new();
            while let Some(d) = chars.next_if(char::is_ascii_digit) {
                s.push(d);
            }
            s
        }
        fn consume_until_closing_bracket(
            chars: &mut Peekable<impl Iterator<Item = char>>,
        ) -> Vec<Element> {
            let mut vec = Vec::new();
            loop {
                match chars
                    .peek()
                    .expect("expected a closing brace, but reached end of input")
                {
                    ']' => {
                        chars.next();
                        return vec;
                    }
                    '0'..='9' => {
                        vec.push(Element::Num(parse_number(chars)));
                    }
                    '[' => {
                        let _ = chars.next();
                        vec.push(Element::List(consume_until_closing_bracket(chars)));
                    }
                    ',' => panic!("expected element before comma"),
                    ' ' => continue,
                    char => panic!("invalid character `{char}`"),
                }
                loop {
                    match chars
                        .peek()
                        .expect("expected a closing brace or comma after element")
                    {
                        ' ' => continue,
                        ',' => {
                            chars.next();
                            break;
                        }
                        ']' => break,
                        char => panic!("invalid character `{char}`"),
                    }
                }
            }
        }

        let mut chars = s.chars().peekable();

        match chars.peek().expect("empty line!") {
            '[' => {
                let _ = chars.next();
                let ele = Element::List(consume_until_closing_bracket(&mut chars));
                match chars.next() {
                    Some(',') => {
                        panic!("unexpected comma (top-level needs to be a list, with '[' and ']')")
                    }
                    Some(']') => {
                        panic!("unexpected ']' (duplicate closing ']', or missing opening '['?")
                    }
                    Some(ch) => {
                        panic!("unexpected character after complete number: `{ch}`");
                    }
                    _ => (),
                }
                ele
            }
            '0'..='9' => {
                let ele = Element::Num(parse_number(&mut chars));
                match chars.next() {
                    Some(',') => {
                        panic!("unexpected comma (top-level needs to be a list, with '[' and ']')")
                    }
                    Some(']') => panic!("unexpected ']' (did you forget the opening '['?"),
                    Some(ch) => {
                        panic!("unexpected character after complete number: `{ch}`");
                    }
                    _ => (),
                }
                ele
            }
            ' ' => unreachable!(),
            ch => panic!("invalid character `{ch}`"),
        }
    }
}
