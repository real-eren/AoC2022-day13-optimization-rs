#![allow(unused)]

use std::{
    borrow::{Borrow, Cow},
    path::Path,
    rc::Rc,
    str::FromStr,
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use day13_compare::{
    input_handling_baseline, logos_lex, manual_lex, naive, naive_slice, prefix_comp_then_logos_lex,
    single_pass_prefix_comp_then_logos_lex, SAMPLE,
};
use duplicate::duplicate;

struct TestData<'a> {
    name: Cow<'a, str>,
    input_fn: Box<dyn Fn() -> Option<Cow<'static, str>>>,
}

fn bench_day13_impls(c: &mut Criterion) {
    let data_set = vec![
        TestData {
            name: "original sample".into(),
            input_fn: Box::new(|| Some(SAMPLE.into())),
        },
        TestData {
            name: "orig sample repeated 1K".into(),
            input_fn: Box::new(|| {
                let mut base = SAMPLE.to_string();
                base.push_str("\n\n");
                base = base.repeat(1000);
                base.truncate(base.len() - 2);
                Some(base.into())
            }),
        },
        TestData {
            name: "single 10kB number, last digit different".into(),
            input_fn: Box::new(|| {
                const S: &str = "1029637485";
                const N: usize = 1000;
                let mut base = S.repeat(N);
                {
                    let base = unsafe { base.as_bytes_mut() };
                    base[0] = b'[';
                    base[base.len() - 2] = b']';
                    base[base.len() - 1] = b'\n';
                }
                base = base.repeat(2);
                base.truncate(base.len() - 2);
                let last_of_first = base.pop().unwrap();
                base.push(if last_of_first == '1' { '0' } else { '1' });
                base.push(']');
                // [12345]\n[12341]
                Some(base.into())
            }),
        },
        TestData {
            name: "single 10kB number, first digit different".into(),
            input_fn: Box::new(|| {
                const S: &str = "1029637485";
                const N: usize = 1000;
                let mut base = S.repeat(N);
                {
                    // [12345]n[12345]
                    let base = unsafe { base.as_bytes_mut() };
                    base[0] = b'[';
                    base[base.len() - 2] = b']';
                    base[base.len() - 1] = b'\n';
                }
                base = base.repeat(2);
                base.pop();
                unsafe {
                    base.as_bytes_mut()[1] = b'5';
                }
                // [12345]n[02345]
                Some(base.into())
            }),
        },
    ];
    fn for_each_file<'fs>(dir: &'fs Path) -> Vec<TestData<'fs>> {
        fn from_file(path: &Path) -> Option<String> {
            match std::fs::read_to_string(path) {
                Ok(string) => Some(string),
                Err(e) => {
                    println!("{}: {e}", path.to_string_lossy());
                    None
                }
            }
        }

        if !dir.exists() {
            eprintln!("was looking for input files in path {}, but it does not exist. (run bench in project root dir, or move input files to './benches/resources/')", dir.to_string_lossy());
            return Vec::new();
        }
        if !dir.is_dir() {
            return Vec::new();
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(dir_entry) => dir_entry,
            Err(e) => {
                eprintln!("while iterating over input files: {e}");
                return Vec::new();
            }
        };
        entries
            .into_iter()
            .filter_map(|entry| {
                match entry {
                    Ok(dir_entry) => Some(dir_entry),
                    Err(e) => {
                        eprintln!("{e}");
                        None
                    }
                }
                .and_then(|dir_entry| {
                    let path = dir_entry.path();
                    if path.is_file() {
                        Some(TestData {
                            name: path
                                .as_path()
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .into_owned()
                                .into(),
                            input_fn: Box::new(move || from_file(path.as_path()).map(Cow::Owned)),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    let mut group = c.benchmark_group("Day13_A");
    for TestData { name, input_fn } in data_set
        .iter()
        .chain(for_each_file(Path::new("./benches/resources/")).iter())
    {
        let Some(ref input) = input_fn() else {
            println!("failed to generate/retrieve input for {name}");
            continue;
        };
        group.throughput(Throughput::Bytes(input.len() as u64));

        duplicate! {
            [
                module_name; [naive::pooled]; [naive::no_pool]; [naive_slice::pooled]; [naive_slice::no_pool]; [manual_lex]; [logos_lex]; [input_handling_baseline]; [single_pass_prefix_comp_then_logos_lex];
            ]
            group.bench_with_input(BenchmarkId::new(stringify!(module_name), name), input, |b, i| {
                b.iter(|| module_name::day13(i))
            });
        }
        duplicate! {
            [chunk_size; [16]; [128]]
            group.bench_with_input(BenchmarkId::new(concat!("prefix_comp_then_logos_lex", chunk_size), name), input, |b, i| {
                b.iter(|| prefix_comp_then_logos_lex::day13::<chunk_size>(i))
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_day13_impls);
criterion_main!(benches);
