use std::{cmp::Ordering, iter::Peekable};

use res_pool::ResPool;

pub const DESCRIPTION: &str = "naive with object pools for Vec and String.";

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

type ListPool = ResPool<Vec<Element>>;
type StringPool = ResPool<String>;

impl Element {
    fn parse(s: &str, list_pool: &mut ListPool, string_pool: &mut StringPool) -> Element {
        let s = s.trim();
        if s.chars().all(|ch| ch.is_ascii_digit()) {
            return Self::Num(s.to_owned());
        }

        fn parse_number(
            chars: &mut Peekable<impl Iterator<Item = char>>,
            string_pool: &mut StringPool,
        ) -> String {
            let mut s = string_pool.withdraw();
            while let Some(d) = chars.next_if(char::is_ascii_digit) {
                s.push(d);
            }
            s
        }
        fn consume_until_closing_bracket(
            chars: &mut Peekable<impl Iterator<Item = char>>,
            list_pool: &mut ListPool,
            string_pool: &mut StringPool,
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

    // tears down this object and returns resources (Vecs, Strings) to the given pools)
    fn scavenge(self, list_pool: &mut ListPool, string_pool: &mut StringPool) {
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

mod res_pool {
    pub struct ResPool<T> {
        items: Vec<T>,
        make_new: Box<dyn FnMut() -> T>,
    }

    impl<T> ResPool<T> {
        pub(super) fn new(supplier: Box<dyn FnMut() -> T>) -> Self {
            ResPool {
                items: Vec::new(),
                make_new: supplier,
            }
        }

        pub(super) fn deposit(&mut self, item: T) {
            self.items.push(item);
        }

        pub(super) fn withdraw(&mut self) -> T {
            self.items.pop().unwrap_or_else(self.make_new.as_mut())
        }
    }
}

pub fn day13(input: &str) -> usize {
    let mut list_pool = ResPool::new(Box::new(Vec::new));
    let mut string_pool = ResPool::new(Box::new(String::new));
    input
        .split("\n\n")
        .map(|chunk| {
            chunk
                .split_once('\n')
                .unwrap_or_else(|| panic!("strange format: {chunk}"))
        })
        .map(|(left, right)| {
            let left = Element::parse(left, &mut list_pool, &mut string_pool);
            let right = Element::parse(right, &mut list_pool, &mut string_pool);
            let cmp = left.cmp(&right);
            left.scavenge(&mut list_pool, &mut string_pool);
            right.scavenge(&mut list_pool, &mut string_pool);
            cmp
        })
        .enumerate()
        .filter_map(|(idx, ord)| if ord.is_lt() { Some(idx + 1) } else { None })
        .sum()
}
