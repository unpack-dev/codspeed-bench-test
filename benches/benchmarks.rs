use std::hint::black_box;

use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use oxc_allocator::Allocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Parser as SwcParser, StringInput, Syntax, lexer::Lexer};

use bench_demo::repeat_string;

const PARSE_SOURCE: &str = include_str!("fixtures/parse_input.js");

fn bench_repeat_string(c: &mut Criterion) {
    c.bench_function("repeat_string_short", |b| {
        b.iter(|| repeat_string("a"));
    });

    c.bench_function("repeat_string_long", |b| {
        b.iter(|| repeat_string("hello world"));
    });
}

fn bench_parsers(c: &mut Criterion) {
    let source_type =
        SourceType::from_path("parse_input.js").expect("fixture path should infer JavaScript");
    let cm: Lrc<SourceMap> = Default::default();
    let mut group = c.benchmark_group("parse");

    group.bench_function("oxc", |b| {
        b.iter(|| {
            let allocator = Allocator::default();
            let ret = OxcParser::new(&allocator, black_box(PARSE_SOURCE), source_type).parse();

            assert!(!ret.panicked, "oxc panicked while parsing fixture");
            assert!(ret.errors.is_empty(), "oxc returned parse errors");

            black_box(ret.program.body.len());
        });
    });

    group.bench_function("swc", |b| {
        b.iter(|| {
            let fm = cm.new_source_file(
                FileName::Custom("parse_input.js".into()).into(),
                black_box(PARSE_SOURCE),
            );
            let lexer = Lexer::new(
                Syntax::Es(Default::default()),
                EsVersion::Es2022,
                StringInput::from(&*fm),
                None,
            );
            let mut parser = SwcParser::new_from(lexer);
            let module = parser.parse_module().expect("swc should parse fixture");

            assert!(parser.take_errors().is_empty(), "swc returned parse errors");

            black_box(module.body.len());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_repeat_string, bench_parsers);
criterion_main!(benches);
