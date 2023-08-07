mod manual_lex;
mod naive;
mod naive_cached;

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
            iters: 3000,
        },
    ];
    println!("Created the following test data:");
    for c in data_set.iter() {
        let name = c.name;
        let size = c.input.len();
        let (adjusted_size, unit) = match size {
            b @ 0..=999 => (b as f64, "bytes"),
            kb @ 1000..=999999 => (kb as f64 / 1000., "kB"),
            mb @ 1000000..=999999999 => (mb as f64 / 1000000., "mB?"),
            gb => (gb as f64 / 1000000000., "gB"),
        };
        let num_pairs = c.input.lines().count() / 3 + 1;
        println!("  {name}: {adjusted_size}{unit}, {num_pairs} pairs")
    }

    let naive = Candidate {
        name: "naive".to_string(),
        desc: naive::DESCRIPTION.to_string(),
        func: naive::day13,
    };
    let naive_cached = Candidate {
        name: "naive cached".to_string(),
        desc: naive_cached::DESCRIPTION.to_string(),
        func: naive_cached::day13,
    };
    let manual_lex = Candidate {
        name: "manual_lex".to_string(),
        desc: manual_lex::DESCRIPTION.to_string(),
        func: manual_lex::day13,
    };
    let candidates: &[Candidate] = &[naive, naive_cached, manual_lex];

    println!("\nPrepared the following impls:");
    for c in candidates.iter() {
        println!("  {}: {}", c.name, c.desc)
    }

    println!("\nstarting to test candidates...");
    for Candidate {
        desc: _,
        name,
        func,
    } in candidates.iter()
    {
        println!("  Testing  impl `{name}`");
        for &TestData {
            name,
            expected,
            input,
            iters,
        } in data_set.iter()
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
                println!("    {name}: {average}s");
            }
        }
        println!("");
    }
}

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
    use crate::{manual_lex, naive, naive_cached, SAMPLE};

    #[test]
    fn naive() {
        assert_eq!(naive::day13(SAMPLE), 13)
    }

    #[test]
    fn naive_cached() {
        assert_eq!(naive_cached::day13(SAMPLE), 13)
    }

    #[test]
    fn manual_lex() {
        assert_eq!(manual_lex::day13(SAMPLE), 13)
    }
}
