//! Char-by-char parser, collects each line into Vec< Vec | String>

use std::{cmp::Ordering, iter::Peekable};

use crate::shared::{
    day13_framework,
    res_pool::{Alloc, GlobalHeapProxy, ResPool},
};

/// creates and drop Vecs and Strings each line (global heap).
pub mod no_pool {
    use super::{day13_generalized, GlobalHeapProxy};

    pub fn day13(input: &str) -> usize {
        let list_pool = &mut GlobalHeapProxy {};
        let string_pool = &mut GlobalHeapProxy {};
        day13_generalized(input, list_pool, string_pool)
    }
}

/// Uses an object pool for the Vecs and Strings
pub mod pooled {
    use super::{day13_generalized, ResPool};

    pub fn day13(input: &str) -> usize {
        let new_list = &mut Vec::new;
        let list_pool = &mut ResPool::new(new_list);

        let new_string = &mut String::new;
        let string_pool = &mut ResPool::new(new_string);

        day13_generalized(input, list_pool, string_pool)
    }
}

fn day13_generalized(
    input: &str,
    list_pool: &mut impl Alloc<Vec<Element>>,
    string_pool: &mut impl Alloc<String>,
) -> usize {
    day13_framework(input, |left, right| {
        let left = Element::parse(left, list_pool, string_pool);
        let right = Element::parse(right, list_pool, string_pool);
        let cmp = left.cmp(&right);
        left.scavenge(list_pool, string_pool);
        right.scavenge(list_pool, string_pool);
        cmp
    })
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
    fn parse(
        s: &str,
        list_pool: &mut impl Alloc<Vec<Element>>,
        string_pool: &mut impl Alloc<String>,
    ) -> Element {
        let s = s.trim();
        if s.chars().all(|ch| ch.is_ascii_digit()) {
            return Self::Num(s.to_owned());
        }

        fn parse_number(
            chars: &mut Peekable<impl Iterator<Item = char>>,
            string_pool: &mut impl Alloc<String>,
        ) -> String {
            let mut s = string_pool.withdraw();
            while let Some(d) = chars.next_if(char::is_ascii_digit) {
                s.push(d);
            }
            s
        }
        fn consume_until_closing_bracket(
            chars: &mut Peekable<impl Iterator<Item = char>>,
            list_pool: &mut impl Alloc<Vec<Element>>,
            string_pool: &mut impl Alloc<String>,
        ) -> Vec<Element> {
            let mut vec = list_pool.withdraw();
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
                        vec.push(Element::Num(parse_number(chars, string_pool)));
                    }
                    '[' => {
                        let _ = chars.next();
                        vec.push(Element::List(consume_until_closing_bracket(
                            chars,
                            list_pool,
                            string_pool,
                        )));
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
                let ele = Element::List(consume_until_closing_bracket(
                    &mut chars,
                    list_pool,
                    string_pool,
                ));
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
                let ele = Element::Num(parse_number(&mut chars, string_pool));
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

    /// tears down this object and returns resources (Vecs, Strings) to the given pools)
    fn scavenge(
        self,
        list_pool: &mut impl Alloc<Vec<Element>>,
        string_pool: &mut impl Alloc<String>,
    ) {
        match self {
            Self::List(mut items) => {
                for ele in items.drain(..) {
                    ele.scavenge(list_pool, string_pool);
                }
                list_pool.deposit(items);
            }
            Element::Num(mut str) => {
                str.truncate(0);
                string_pool.deposit(str);
            }
        }
    }
}
