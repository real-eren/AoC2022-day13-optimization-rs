//! Like [crate::naive], but uses &str instead of String

use std::{
    cmp::Ordering,
    iter::Peekable,
    mem::{align_of, forget, size_of},
    str::CharIndices,
};

use crate::shared::{
    day13_framework,
    res_pool::{self, Alloc},
};

/// Creates and drops Vecs each line.
pub mod no_pool {
    use super::{day13_generalized, res_pool::GlobalHeapProxy};

    pub fn day13(input: &str) -> usize {
        let list_pool = &mut GlobalHeapProxy {};
        day13_generalized(input, list_pool)
    }
}
/// Uses an object pool for the Vecs.
pub mod pooled {
    use super::{day13_generalized, res_pool::ResPool};

    pub fn day13(input: &str) -> usize {
        let new_list = &mut Vec::new;
        let list_pool = &mut ResPool::new(new_list);

        day13_generalized(input, list_pool)
    }
}

fn day13_generalized<'a>(input: &str, list_pool: &mut impl Alloc<Vec<Element<'a>>>) -> usize {
    day13_framework(input, |left, right| {
        let left = Element::parse(left, list_pool);
        let right = Element::parse(right, list_pool);
        let cmp = left.cmp(&right);
        left.scavenge(list_pool);
        right.scavenge(list_pool);
        cmp
    })
}

#[derive(PartialEq, Eq, Debug)]
enum Element<'a> {
    Num(&'a str),
    List(Vec<Element<'a>>),
}

impl<'a> PartialOrd for Element<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Element<'a> {
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

impl<'a> Element<'a> {
    fn parse<'pool>(s: &'a str, list_pool: &mut impl Alloc<Vec<Element<'pool>>>) -> Element<'a> {
        let s = s.trim();
        if s.chars().all(|ch| ch.is_ascii_digit()) {
            return Self::Num(s);
        }

        fn parse_number<'a>(chars: &mut Peekable<CharIndices>, source: &'a str) -> &'a str {
            let start = chars.next().unwrap().0;
            let mut end = start;
            while let Some((idx, _)) = chars.next_if(|(_, c)| c.is_ascii_digit()) {
                end = idx;
            }
            source.split_at(end + 1).0.split_at(start).1
        }
        fn consume_until_closing_bracket<'b, 'source>(
            chars: &mut Peekable<CharIndices>,
            source: &'source str,
            list_pool: &mut impl Alloc<Vec<Element<'b>>>,
        ) -> Vec<Element<'source>> {
            let mut vec = launder(list_pool.withdraw());
            loop {
                match chars
                    .peek()
                    .expect("expected a closing brace, but reached end of input")
                    .1
                {
                    ']' => {
                        chars.next();
                        return vec;
                    }
                    '0'..='9' => {
                        vec.push(Element::Num(parse_number(chars, source)));
                    }
                    '[' => {
                        let _ = chars.next();
                        vec.push(Element::List(consume_until_closing_bracket(
                            chars, source, list_pool,
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
                        .1
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

        let mut chars = s.char_indices().peekable();

        match chars.peek().expect("empty line!").1 {
            '[' => {
                let _ = chars.next();
                let ele = Element::List(consume_until_closing_bracket(&mut chars, s, list_pool));
                match chars.next().map(|a| a.1) {
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
                let ele = Element::Num(parse_number(&mut chars, s));
                match chars.next().map(|a| a.1) {
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
    fn scavenge<'b>(self, list_pool: &mut impl Alloc<Vec<Element<'b>>>) {
        match self {
            Self::List(mut items) => {
                for ele in items.drain(..) {
                    ele.scavenge(list_pool);
                }
                let items = launder(items);
                list_pool.deposit(items);
            }
            Element::Num(_) => {}
        }
    }
}

/// Intended use: ignore lifetime of Vec<Element<'a>> when vec is empty before *and* after
/// borrowing
fn launder<Old, New>(mut old: Vec<Old>) -> Vec<New> {
    assert!(old.is_empty());
    assert_eq!(align_of::<Old>(), align_of::<New>());
    assert_eq!(size_of::<Old>(), size_of::<New>());
    unsafe {
        let capacity = old.capacity();
        let len = old.len();
        let ptr = old.as_mut_ptr();
        forget(old);
        Vec::from_raw_parts(ptr as *mut New, len, capacity)
    }
}
