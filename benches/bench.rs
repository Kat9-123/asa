use asa::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;

fn runtimes(c: &mut Criterion) {
    let contents = fs::read_to_string("subleq/sublib/tests/jumpiftest.sbl").unwrap();

    let (mut mem, _) =
        assembler::assemble(&contents, "subleq/sublib/tests/jumpiftest.sbl".to_owned());
    c.bench_function("slow", |b| {
        b.iter(|| interpreter::interpret(&mut mem, black_box(false)))
    });
    c.bench_function("fast", |b| b.iter(|| interpreter::interpret_fast(&mut mem)));
}

fn assembler(c: &mut Criterion) {
    let contents = fs::read_to_string("subleq/sublib/tests/jumpiftest.sbl").unwrap();
    c.bench_function("assembler", |b| {
        b.iter(|| assembler::assemble(&contents, "subleq/sublib/tests/jumpiftest.sbl".to_owned()))
    });
}

criterion_group!(benches, runtimes, assembler);
criterion_main!(benches);
