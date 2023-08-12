#![allow(unused)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use day13_compare::{
    input_handling_baseline, logos_lex, manual_lex, naive, naive_slice, prefix_comp_then_logos_lex,
    SAMPLE,
};
use duplicate::duplicate;

struct TestData<'a> {
    name: &'a str,
    input: &'a str,
}

fn bench_day13_impls(c: &mut Criterion) {
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
        },
        TestData {
            name: "orig sample repeated 1K",
            input: repeated_sample.as_str(),
        },
    ];
    let mut group = c.benchmark_group("Day13_A");
    for TestData { name, input } in data_set.iter() {
        duplicate! {
            [
                module_name; [naive::pooled]; [naive::no_pool]; [naive_slice::pooled]; [naive_slice::no_pool]; [manual_lex]; [logos_lex]; [input_handling_baseline];
            ]
            group.bench_with_input(BenchmarkId::new(stringify!(module_name), name), input, |b, i| {
                b.iter(|| module_name::day13(*i))
            });
        }
        duplicate! {
            [chunk_size; [16]; [128]]
            group.bench_with_input(BenchmarkId::new(concat!("prefix_comp_then_logos_lex", stringify!(chunk_size)), name), input, |b, i| {
                b.iter(|| prefix_comp_then_logos_lex::day13::<chunk_size>(*i))
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_day13_impls);
criterion_main!(benches);
