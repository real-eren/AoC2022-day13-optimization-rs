//! Dependency for day13 implementations.
use std::cmp::Ordering;

/// Outline of a solution - extracts pairs and passes them to the given line comparator
#[inline(always)]
pub(crate) fn day13_framework(
    mut input: &str,
    mut line_comparator: impl FnMut(&str, &str) -> Ordering,
) -> usize {
    let mut count = 0;
    let mut idx = 1;
    while !input.is_empty() {
        let Some((left, rem)) = input.split_once('\n') else {break};
        let (right, rem) = rem.split_once('\n').unwrap_or((rem, ""));

        if line_comparator(left, right).is_lt() {
            count += idx;
        }

        input = rem.trim_start_matches('\n');
        idx += 1;
    }
    count
}

/// Object pool that moves the resources.
///
/// See [Alloc] for the relevant trait.
/// # Impls
/// See [GlobalHeapProxy] for a 0-cost pool that simply `new`s and `drop`s the items.
/// See [ResPool] for
pub(crate) mod res_pool {
    /// A trait for behaving as an allocator for a concrete type T.
    pub trait Alloc<T> {
        /// Transfers ownership of `item` from the caller to [self]
        fn deposit(&mut self, item: T);
        /// Transfer ownership of some instance of T to the caller
        ///
        /// the instance may be freshly constructed
        fn withdraw(&mut self) -> T;
    }

    /// A ZST that constructs and drops the [Default] resource on-demand
    pub struct GlobalHeapProxy();

    impl<Resource: Default> Alloc<Resource> for GlobalHeapProxy {
        fn deposit(&mut self, _: Resource) {
            // drop item
        }

        fn withdraw(&mut self) -> Resource {
            Resource::default()
        }
    }

    /// A stack of T, with a closure to create new instances as needed
    ///
    /// The intended use is to `withdraw()` when you need a new instance,
    /// and to `deposit(T)` that instance when you are done with it.
    /// Because the values are `moved`, there is no requirement that the item be returned to the
    /// same ResPool - you could return to a different ResPool or just Drop the item.
    pub struct ResPool<'a, T> {
        items: Vec<T>,
        make_new: &'a mut dyn FnMut() -> T,
    }

    impl<'a, T> ResPool<'a, T> {
        pub(crate) fn new(supplier: &'a mut dyn FnMut() -> T) -> Self {
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
