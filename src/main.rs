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
    println!("Hello, world!");
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
    let candidates: &[Candidate] = &[naive, naive_cached];

    println!("\nCreated the following test data:");

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
