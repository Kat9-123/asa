use asa::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;

fn runtimes(c: &mut Criterion) {
    let path = "./subleq/sublib/tests/JumpIfTest.sbl";
    let contents = fs::read_to_string(path).unwrap();

    let (mut mem, toks) = assembler::assemble(&contents, path.to_owned());

    c.bench_function("fast", |b| {
        b.iter(|| runtimes::interpreter::interpret(&mut mem))
    });
    c.bench_function("debugger", |b| {
        b.iter(|| runtimes::debugger::run_with_debugger(&mut mem, &toks, black_box(false)))
    });
}

fn assembler(c: &mut Criterion) {
    let contents = fs::read_to_string("./subleq/Sublib/tests/JumpIfTest.sbl").unwrap();
    c.bench_function("assembler", |b| {
        b.iter(|| {
            assembler::assemble(
                &contents,
                "./subleq/Sublib/tests/ControlTest.sbl".to_owned(),
            )
        })
    });
}

criterion_group!(benches, assembler, runtimes);
criterion_main!(benches);
