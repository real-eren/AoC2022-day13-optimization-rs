An informal case study on the performance of various approaches to solving the programming puzzle posed in Advent of Code 2022 Day 13, part a.

Major points can be navigated with the git tags
- line-splitting-v1
- line-splitting-v2
- line-splitting-v2-modded-lex

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

## Line splitting
The implementations shared this portion of the code, which extracts the pairs of lines from the input, and differed only in how they compared the two lines.
The performance of just this code can be approximated by `./src/input_handling_baseline.rs`.

### v1
```rust
use std::cmp::Ordering;

/// Outline of a solution - extracts pairs and passes them to the given line comparator
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
Here, the "\n\n" split iterator was replaced with a manual while loop that fetches one line at a time. 
The key difference is that the framework only scans each line once, instead of one-and-a-half times.
As discussed later, this change was not as impactful for the slower implementations (of `line_comparator`), but was significant for the leaner implementations such as `prefix_comp`
It is also possible that splitting on a single character is easier to optimize than splitting on a string, though I have not investigated this.

Another interesting point is that this approach is easier to adapt to streamed input (readers)
```rust
use std::cmp::Ordering;

/// Outline of a solution - extracts pairs and passes them to the given line comparator
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

TODO

# Findings
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
RUSTFLAGS="-g" CARGO_PROFILE_BENCH_DEBUG="true" cargo bench --no-run

# find the newly created executable with the prefix "day13_impls" (should be in the console output of the above cargo command)
BENCH="./target/release/deps/day13_impls-43b168dda538aecb"
valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --cache-sim=yes $BENCH --bench --profile 10
```

### [Flamegraph](https://github.com/flamegraph-rs/flamegraph)
This uses `perf` to (basically) sample function calls, and can be used to identify the functions where time is being spent.

```
# follow earlier instructions for building and finding the bench executable
flamegraph -- $BENCH --bench --profile-time 10
```
I would strongly recommend running the above benchmark with a single implementation selected, and possibly a single input.
ex:
to run just the 'logos_lex' impl on all the input: `flamegraph -- $BENCH --bench --profile-time 10 "Day13_A/logos_lex"`
to run just the 'logos_lex' impl on just the 'original input': `flamegraph -- $BENCH --bench --profile-time 10 "Day13_A/logos_lex/original input"`

Unfortunately, the criterion harness adds a bit of noise to the flamegraph, so to focus on the relevant portions of the flamegraph, navigate to these in order:
1) `day13_impls::main`
2) `criterion::routine::Routine::profile`
3) `<criterion::routine::Function<M,F,T> as criterion::routine::Routine<M,T>>::bench`
4) `day13_compare::logos_lex::day13`, where logos_lex is the particular implementation you want to investigate

Larger values of --profile-time will also help the `day13` to stand out (over the warm_up and set-up functions). For instance, passing 40 (seconds) made the day13 function account for 91% of the samples.

