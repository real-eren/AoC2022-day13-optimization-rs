mod logos_lex;
mod manual_lex;
mod naive;
mod prefix_comp_then_logos_lex;
mod shared;

use std::time::Instant;

struct Candidate {
    name: String,
    desc: String,
    func: fn(&str) -> usize,
}

struct TestData<'a> {
    name: &'a str,
    input: &'a str,
    expected: usize,
    iters: usize,
}

// TODO: replace with `criterion` crate
fn main() {
    println!("Starting AoC.2022.day13 benchmark");

    println!("constructing test data...");

    let repeated_sample = {
        let mut base = SAMPLE.to_string();
        base.push_str("\n\n");
        base = base.repeat(1000);
        base.truncate(base.len() - 2);
        base
    };
    let data_set: &[TestData] = &[
        TestData {
            name: "original sample",
            input: SAMPLE,
            expected: 13,
            iters: 100000,
        },
        TestData {
            name: "orig sample repeated 1K",
            input: repeated_sample.as_str(),
            expected: 15997000,
            iters: 300,
        },
    ];
    println!("Created the following test data:");
    for (idx, c) in data_set.iter().enumerate() {
        let name = c.name;
        let size = c.input.len();
        let iters = c.iters;
        let size = match size {
            b @ 0..=999 => format!("{b}B"),
            kb @ 1000..=999999 => format!("{:.3}kB", (kb as f64 / 1000.)),
            mb @ 1000000..=999999999 => format!("{:.3}mB", mb as f64 / 1000000.),
            gb => format!("{:.3}gB", gb as f64 / 1000000000.),
        };
        let num_pairs = c.input.lines().count() / 3 + 1;
        println!(
            "  {}) {name}:
    {size},
    {num_pairs} pairs,
    {iters} iterations",
            idx + 1
        )
    }

    let naive = Candidate {
        name: "naive".to_string(),
        desc: naive::no_pool::DESCRIPTION.to_string(),
        func: naive::no_pool::day13,
    };
    let naive_cached = Candidate {
        name: "naive cached".to_string(),
        desc: naive::pooled::DESCRIPTION.to_string(),
        func: naive::pooled::day13,
    };
    let manual_lex = Candidate {
        name: "manual_lex".to_string(),
        desc: manual_lex::DESCRIPTION.to_string(),
        func: manual_lex::day13,
    };
    let logos_lex = Candidate {
        name: "logos_lex".to_string(),
        desc: logos_lex::DESCRIPTION.to_string(),
        func: logos_lex::day13,
    };
    let skip_prefix_then_lex_128 = Candidate {
        name: "skip_prefix_then_lex".to_string(),
        desc: prefix_comp_then_logos_lex::DESCRIPTION.to_string(),
        func: prefix_comp_then_logos_lex::day13::<128>,
    };
    let skip_prefix_then_lex_16 = Candidate {
        name: "skip_prefix_then_lex".to_string(),
        desc: prefix_comp_then_logos_lex::DESCRIPTION.to_string(),
        func: prefix_comp_then_logos_lex::day13::<16>,
    };
    let candidates: &[Candidate] = &[
        naive,
        naive_cached,
        manual_lex,
        logos_lex,
        skip_prefix_then_lex_128,
        skip_prefix_then_lex_16,
    ];

    println!("\nPrepared the following impls:");
    for (idx, c) in candidates.iter().enumerate() {
        println!(
            "  {}) {}: {}",
            char::from_u32('a' as u32 + idx as u32).unwrap(),
            c.name,
            c.desc
        )
    }

    println!("\nstarting to test candidates...");
    for (
        idx,
        Candidate {
            desc: _,
            name,
            func,
        },
    ) in candidates.iter().enumerate()
    {
        println!(
            "  {}) `{name}`",
            char::from_u32(idx as u32 + 'a' as u32).unwrap()
        );
        for (
            idx,
            &TestData {
                name,
                expected,
                input,
                iters,
            },
        ) in data_set.iter().enumerate()
        {
            let now = Instant::now();
            let answer = func(input);
            for _ in 1..iters {
                func(input);
            }
            let elapsed = now.elapsed().as_secs_f64();
            let average = elapsed / iters as f64;
            if answer != expected {
                println!("    failed `{name}`! got {answer}, expected {expected}");
            } else {
                println!("    {}) {name}: {average:.9}s", idx + 1);
            }
        }
        println!();
    }
}

/// input provided in the AoC entry
const SAMPLE: &str = "[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";

#[cfg(test)]
mod tests {
    use crate::{logos_lex, manual_lex, naive, prefix_comp_then_logos_lex, SAMPLE};

    #[test]
    fn naive() {
        assert_eq!(naive::no_pool::day13(SAMPLE), 13)
    }

    #[test]
    fn naive_cached() {
        assert_eq!(naive::pooled::day13(SAMPLE), 13)
    }

    #[test]
    fn manual_lex() {
        assert_eq!(manual_lex::day13(SAMPLE), 13)
    }

    #[test]
    fn logos_lex() {
        assert_eq!(logos_lex::day13(SAMPLE), 13)
    }

    #[test]
    fn skip_prefix_then_lex() {
        assert_eq!(prefix_comp_then_logos_lex::day13::<128>(SAMPLE), 13)
    }
}
