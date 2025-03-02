use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use sol::{lexer::Lexer, parser::Parser, typechecker::Typechecker};

// FIXME: errors are causing the benchmark to basically increase in memory forever

fn criterion_benchmark(c: &mut Criterion) {
    let input = include_str!("./input.sol");

    c.bench_function("lexer", |b| {
        b.iter_batched(
            || Lexer::new(0, input),
            move |lexer| {
                let _ = lexer.collect::<Vec<_>>();
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("parser", |b| {
        b.iter_batched(
            || {
                let lexer = Lexer::new(0, input);
                Parser::new(lexer, input)
            },
            move |parser| {
                // parser returning result means error builds infinitely and ooms :)
                let _ = parser.collect::<Vec<_>>();
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("typechecker", |b| {
        b.iter_batched(
            || {
                let lexer = Lexer::new(0, input);
                Parser::new(lexer, input)
                    .map(|s| s.unwrap())
                    .collect::<Vec<_>>()
            },
            move |statements| Typechecker::default().check(&statements),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
