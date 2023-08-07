use std::{cmp::Ordering, iter::Peekable};

use self::res_pool::Alloc;

pub mod no_pool {
    use super::{day13_generalized, res_pool::GlobalHeapProxy};

    pub const DESCRIPTION: &str = "Parses each line into Vec< Vec | String>";

    pub fn day13(input: &str) -> usize {
        let list_pool = &mut GlobalHeapProxy {};
        let string_pool = &mut GlobalHeapProxy {};
        day13_generalized(input, list_pool, string_pool)
    }
}
pub mod pooled {
    use super::{day13_generalized, res_pool::ResPool};

    pub const DESCRIPTION: &str = "naive with object pools for Vec and String.";

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
    input
        .split("\n\n")
        .map(|chunk| {
            chunk
                .split_once('\n')
                .unwrap_or_else(|| panic!("strange format: {chunk}"))
        })
        .map(|(left, right)| {
            let left = Element::parse(left, list_pool, string_pool);
            let right = Element::parse(right, list_pool, string_pool);
            let cmp = left.cmp(&right);
            left.scavenge(list_pool, string_pool);
            right.scavenge(list_pool, string_pool);
            cmp
        })
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

mod res_pool {
    pub trait Alloc<T> {
        fn deposit(&mut self, item: T);
        fn withdraw(&mut self) -> T;
    }

    pub struct GlobalHeapProxy();

    impl<Resource: Default> Alloc<Resource> for GlobalHeapProxy {
        fn deposit(&mut self, _: Resource) {
            // drop item
        }

        fn withdraw(&mut self) -> Resource {
            Resource::default()
        }
    }

    pub struct ResPool<'a, T> {
        items: Vec<T>,
        make_new: &'a mut dyn FnMut() -> T,
    }

    impl<'a, T> ResPool<'a, T> {
        pub(super) fn new(supplier: &'a mut dyn FnMut() -> T) -> Self {
            ResPool {
                items: Vec::new(),
                make_new: supplier,
            }
        }
    }

    impl<'a, T> Alloc<T> for ResPool<'a, T> {
        fn deposit(&mut self, item: T) {
            self.items.push(item);
        }

        fn withdraw(&mut self) -> T {
            self.items.pop().unwrap_or_else(&mut self.make_new)
        }
    }
}
