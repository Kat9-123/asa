use asa::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;

fn runtimes(c: &mut Criterion) {
    let path = "./subleq/sublib/tests/JumpIfTest.sbl";
    let contents = fs::read_to_string(path).unwrap();

    let (mut mem, toks) = assembler::assemble(&contents, path.to_owned());

    c.bench_function("normal", |b| {
        b.iter(|| runtimes::interpreter::interpret(&mut mem))
    });

    c.bench_function("debugger,", |b| {
        b.iter(|| runtimes::debugger::run_with_debugger(&mut mem, &toks))
    });
}

fn assembler(c: &mut Criterion) {
    let contents = fs::read_to_string("./subleq/Sublib/tests/JumpIfTest.sbl").unwrap();
    c.bench_function("assembler normal", |b| {
        b.iter(|| {
            assembler::assemble(
                &contents,
                "./subleq/Sublib/tests/ControlTest.sbl".to_owned(),
            )
        })
    });
    let contents = fs::read_to_string("./subleq/large.sbl").unwrap();
    c.bench_function("assembler large", |b| {
        b.iter(|| assembler::assemble(&contents, "./subleq/large.sbl".to_owned()))
    });
}

criterion_group!(benches, assembler, runtimes);
criterion_main!(benches);
