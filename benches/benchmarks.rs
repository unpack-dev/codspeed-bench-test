use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};

use bench_demo::repeat_string;

fn bench_repeat_string(c: &mut Criterion) {
    c.bench_function("repeat_string_short", |b| {
        b.iter(|| repeat_string("a"));
    });

    c.bench_function("repeat_string_long", |b| {
        b.iter(|| repeat_string("hello world"));
    });
}

criterion_group!(benches, bench_repeat_string);
criterion_main!(benches);
