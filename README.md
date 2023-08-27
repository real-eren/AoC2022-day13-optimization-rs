An informal case study on the performance of various approaches to solving the programming puzzle posed in Advent of Code 2022 Day 13, part a.

Major points can be navigated with the git tags
- line-splitting-v1
- line-splitting-v2
- line-splitting-v2-modded-lex+single-pass
- single_pass

# Problem
The full prompt can be found at https://adventofcode.com/2022/day/13

In short, given an input like the below,
```
[1,1,3,1,1]
[1,1,5,1,1]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]
```
we want to identify the pairs of lines where the two lines are in lexicographical order (like a dictionary)

# Implementations
Brief descriptions of components of the various implementations.

## Line splitting
Most of the implementations shared this portion of the code, which extracts the pairs of lines from the input, and differed only in how they compared the two lines.
The performance of just this code can be approximated by `./src/input_handling_baseline.rs`.

### v1
```rust
use std::cmp::Ordering;

pub(crate) fn day13_framework(
    input: &str,
    mut line_comparator: impl FnMut(&str, &str) -> Ordering,
) -> usize {
    input
        .split("\n\n")
        .map(|chunk| {
            chunk
                .split_once('\n')
                .unwrap_or_else(|| panic!("strange format: {chunk}"))
        })
        .map(|(l, r)| line_comparator(l, r))
        .enumerate()
        .filter_map(|(idx, ord)| if ord.is_lt() { Some(idx + 1) } else { None })
        .sum()
}
```

### v2
Here, the `"\n\n"` split iterator was replaced with a while loop that fetches one line at a time. 
The key difference is that the framework only scans each line once, instead of one-and-a-half times.
As discussed later, this change was not as impactful for the slower implementations (of `line_comparator`), but was significant for the leaner implementations such as `prefix_comp`
It is also possible that splitting on a single character is easier to optimize than splitting on a string, though I have not investigated this.

Another interesting point is that this approach is easier to adapt to streamed input (readers)
```rust
use std::cmp::Ordering;

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
    return count;
}
```

## Line compare

### Naive Parsing (./src/naive.rs and ./src/naive_slice.rs)
Character-by-character parsing. 
The entire parse tree is constructed for both lines before the elements are compared.
```rust
enum Element {
    Num(String),
    List(Vec<Element>),
}
```
There are 4 implementations:
2 with object pools (and 2 without),
and 2 with string slices (and 2 without).

### Lexing (./src/manual_lex.rs and ./src/logos_lex.rs)
Instead of creating and storing the parse trees, the individual lexemes from both lines are compared one-by-one.
A small (and constant) amount of extra space is used to track the 'depth' of braces.

1 implementation w/ character-by-character lexing and another using the `Logos` lexer generator crate.
#### Logos Lexer definition
```rust
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
```
The `LBraces` token groups together consecutive `[`s because this save some cycles on deeply nested lists witout pessimizing normal inputs, and does not complicate the implementation.

### Prefix Compare + Lexing (./src/prefix_comp_then_logos_lex.rs)
This approach assumes the input is syntactically valid.
Byte-wise compare the left and right lines until either one line ends or a difference is found. 
Then a lexer is ran on the remainders until either a decision is made or the two lines reach an equivalence point again.
Then the process repeats.

### 'Single-pass' Prefix Compare + ... (./src/single_pass_prefix_comp_then_logos_lex.rs)
Similar to the previous one, however the right line is not sliced before the comparison.
Instead, the compare subroutine takes in the `left` and *`rem`* string slices,
reads no further than the first new line in `rem`,
and returns the number of bytes into `rem` that were inspected.
The outer loop then searches for the end of the right line (the first line in rem) starting from that position.
```rust
fn compare_first_line(left: &str, rem: &str) -> (Ordering, usize) {
    // ...
}
```

## Object Pooling
. The values were moved, as opposed to borrowed w/ `&mut`, for ease of implementation.
```rust
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
```

# Findings

## Performance Comparisons
TODO

## Object pooling

This is particularly impactful for inputs with many lines.

id to impl:
| id # | number repr | Object Pool  |
|------|-------------|------------- |
| 1    | String      | y            |
| 2    | String      | n            |
| 3    | &str        | y            |
| 4    | &str        | n            |

| input_name \ impl name       | 1      | 2      | 3      |  4     | time unit
|------------------------------|--------|--------|--------|--------|--------
|original sample               | 2.8694 | 3.8687 | 2.1769 | 2.4931 | µs
|orig sample repeated 1K       | 2.0583 | 4.0140 | 1.7664 | 2.6523 | ms
|single 10kB number, last diff | 38.525 | 41.079 | 31.785 | 33.914 | µs
|single 10kB number, first diff| 38.547 | 41.694 | 31.786 | 34.305 | µs
|file-long_mixed_lines         | 6.8266 | 12.440 | 5.5249 | 7.7700 | µs
|file-right_longer             | 4.7476 | 7.3911 | 3.7543 | 4.6300 | µs
|file-left_longer              | 4.7119 | 7.4186 | 3.7193 | 4.7910 | µs
|file-alternating_deep_nesting | 10.652 | 20.363 | 9.7468 | 13.818 | µs

That same data, normalized against column 2 (`naive::no_pool`) (String + no pool)
| input_name \ impl name       | 1     | 2   | 3     | 4     |
|------------------------------|-------|-----|-------|-------|
|original sample               | 0.742 | 1.0 | 0.563 | 0.644 |
|orig sample repeated 1K       | 0.513 | 1.0 | 0.441 | 0.669 |
|single 10kB number, last diff | 0.938 | 1.0 | 0.774 | 0.826 |
|single 10kB number, first diff| 0.925 | 1.0 | 0.762 | 0.823 |
|file-long_mixed_lines         | 0.549 | 1.0 | 0.444 | 0.624 |
|file-right_longer             | 0.642 | 1.0 | 0.508 | 0.626 |
|file-left_longer              | 0.635 | 1.0 | 0.501 | 0.646 |
|file-alternating_deep_nesting | 0.523 | 1.0 | 0.479 | 0.679 |
|geometric mean                | 0.666 | 1.0 | 0.546 | 0.688 |

## Impact of multiple passes over strings
TODO


### line-splitting V1 vs V2
TODO

# Benchmarking
[criterion](https://docs.rs/criterion/latest/criterion/) is used for conducting the benchmarks.  

Here is a basic command that will benchmark all the implementations in the current commit,
with the bracketed text being an optional argument to record the data under a given name.
`cargo bench --bench day13_impls -- [--save-baseline put_name_here]`
More options, such as running a subset of the cases or selecting an existing baseline (as opposed to the default, the most recent run) can be viewed at [the documentation for Criterion](https://bheisler.github.io/criterion.rs/book/user_guide/command_line_options.html)

for quick reference, this will run the Criterion benchmark suite, but only the 'lex_logos' implementation will be tested, against just the 'original input' input.
`cargo bench --bench day13_impls -- [other Criterion opts] "Day13_A/lex_logos/original input"`

Here is the (subtly different!) equivalent for the other commands below that take an executable, 
`{profile command} [other profiler options] {path/to/bench/executable} --bench "Day13_A/lex_logos/original input"`
Specific examples for various profilers are found in their corresponding sections below =).


Additionally passing the options `--format terse --quiet` result in output like below:
```
Benchmarking Day13_A/input_handling_baseline/file-long_mixed_lines: Collecting 100 samples in e
Day13_A/input_handling_baseline/file-long_mixed_lines
                        time:   [248.34 ns 249.27 ns 250.17 ns]
                        thrpt:  [2.8703 GiB/s 2.8806 GiB/s 2.8914 GiB/s]
Benchmarking Day13_A/input_handling_baseline/file-right_longer: Collecting 100 samples in estim
Day13_A/input_handling_baseline/file-right_longer
                        time:   [166.27 ns 175.70 ns 185.67 ns]
                        thrpt:  [2.4678 GiB/s 2.6078 GiB/s 2.7559 GiB/s]
Benchmarking Day13_A/input_handling_baseline/file-left_longer: Collecting 100 samples in estima
Day13_A/input_handling_baseline/file-left_longer
                        time:   [161.64 ns 162.97 ns 164.53 ns]
                        thrpt:  [2.7850 GiB/s 2.8117 GiB/s 2.8347 GiB/s]
Benchmarking Day13_A/input_handling_baseline/file-alternating_deep_nesting: Warming up for 3.00
Benchmarking Day13_A/input_handling_baseline/file-alternating_deep_nesting: Collecting 100 samp
Day13_A/input_handling_baseline/file-alternating_deep_nesting
                        time:   [135.69 ns 135.96 ns 136.26 ns]
                        thrpt:  [4.0599 GiB/s 4.0690 GiB/s 4.0770 GiB/s]
```
Note that the last entry has an extra line.


## Profiling
Below are some tools and corresponding commands I have used to profile the code while benchmarking. (Tested on Linux, x86-64, Ubuntu 22.04, kernel 6.2._-generic).
Each tool has installation instructions, which can be found through the links.
Note that `perf`, and by extension `flamegraph`, requires access to perf events, which is often blocked for non-priviliged users. On my machine, setting the value to `2` (default was `4`) seemed to be enough to get `perf` working.

### [Heaptrack](https://github.com/KDE/heaptrack)
```
RUSTFLAGS="-g" CARGO_PROFILE_BENCH_DEBUG="true" cargo bench --bench day13_impls --no-run
# find the newly created executable with the prefix "day13_impls"
BENCH="./target/release/deps/day13_impls-43b168dda538aecb"
heaptrack $BENCH --bench --profile-time 10 Day13_A/prefix_comp
```

### [Valgrind](https://valgrind.org/)
Here is a helpful intro to [Rust + Criterion + Valgrind](https://nickb.dev/blog/guidelines-on-benchmarking-and-rust/)
```
# build bench profile, but don't run
# as of this writing, cargo build --bench was *not* equivalent
RUSTFLAGS="-g" CARGO_PROFILE_BENCH_DEBUG="true" cargo bench --no-run

# find the newly created executable with the prefix "day13_impls" (should be in the console output of the above cargo command)
BENCH="./target/release/deps/day13_impls-43b168dda538aecb"
valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --cache-sim=yes $BENCH --bench --profile 10
```

### [Perf](https://perf.wiki.kernel.org/index.php/Main_Page)
(I installed it through the linux-tools-`uname -r` package in apt)
```
# follow earlier instructions for building and finding the bench executable
perf stat -d -d -d -- $BENCH  --bench --profile-time 10 Day13_A/to
```

### [Flamegraph](https://github.com/flamegraph-rs/flamegraph)
This uses `perf` to (basically) sample function calls, and can be used to identify the functions where time is being spent.

```
# follow earlier instructions for building and finding the bench executable
flamegraph -- $BENCH --bench --profile-time 10
```
Because a lot of data will be captured (output file defaults to "./perf.data"), I would strongly recommend running the above benchmark with a single implementation selected, and possibly a single input.
ex:
to run just the 'logos_lex' impl on all the input: `flamegraph -- $BENCH --bench --profile-time 10 "Day13_A/logos_lex"`
to run just the 'logos_lex' impl on just the 'original input': `flamegraph -- $BENCH --bench --profile-time 10 "Day13_A/logos_lex/original input"`

Unfortunately, the criterion harness adds a bit of noise to the flamegraph, so to focus on the relevant portions of the flamegraph, navigate to these in order:
1) `day13_impls::main`
2) `criterion::routine::Routine::profile`
3) `<criterion::routine::Function<M,F,T> as criterion::routine::Routine<M,T>>::bench`
4) `day13_compare::logos_lex::day13`, where logos_lex is the particular implementation you want to investigate

Larger values of --profile-time will also help the `day13` to stand out (over the warm_up and set-up functions).
For instance, passing 40 (seconds) made the day13 function account for 91% of the samples.

# Related Media
Some resources I found informative and helpful:
["Performance Matters" by Emery Berger](https://www.youtube.com/watch?v=r-TLSBdHe1A)
["Parsing JSON Really Quickly: Lessons Learned"](https://www.youtube.com/watch?v=wlvKAT7SZIQ)
https://nickb.dev/blog/guidelines-on-benchmarking-and-rust/

