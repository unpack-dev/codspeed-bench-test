use std::{
    fs,
    hint::black_box,
    path::{Path, PathBuf},
};

use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use oxc_allocator::Allocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use rayon::prelude::*;
use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Parser as SwcParser, StringInput, Syntax, lexer::Lexer};

use bench_demo::repeat_string;

// Vendored from typescript@5.9.3/lib/typescript.js.
const PARSE_SOURCE_FILENAME: &str = "typescript.js";
const PARSE_SOURCE: &str = include_str!("fixtures/typescript.js");
const THREEJS_ESM_ROOT: &str = "benches/fixtures/threejs";
const THREEJS_ESM_FILE_COUNT: usize = 1_115;

struct FixtureSource {
    path: String,
    source: String,
    source_type: SourceType,
}

fn parse_oxc_source(path: &str, source: &str, source_type: SourceType) -> usize {
    let allocator = Allocator::default();
    let ret = OxcParser::new(&allocator, source, source_type).parse();

    assert!(!ret.panicked, "oxc panicked while parsing {path}");
    assert!(
        ret.errors.is_empty(),
        "oxc returned parse errors for {path}"
    );

    ret.program.body.len()
}

fn parse_swc_source(path: &str, source: &str) -> usize {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom(path.into()).into(),
        black_box(source.to_owned()),
    );
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        EsVersion::EsNext,
        StringInput::from(&*fm),
        None,
    );
    let mut parser = SwcParser::new_from(lexer);
    let module = parser.parse_module().expect("swc should parse fixture");

    assert!(
        parser.take_errors().is_empty(),
        "swc returned parse errors for {path}"
    );

    module.body.len()
}

fn collect_fixture_paths(dir: &Path, paths: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("three.js fixture directory should exist: {}", dir.display()))
        .map(|entry| entry.expect("three.js fixture directory entry should be readable"))
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name.starts_with("._") {
            continue;
        }

        if path.is_dir() {
            collect_fixture_paths(&path, paths);
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "js") {
            paths.push(path);
        }
    }
}

fn load_threejs_esm_fixtures() -> Vec<FixtureSource> {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(PathBuf::from(THREEJS_ESM_ROOT));
    let mut fixture_paths = Vec::new();

    collect_fixture_paths(&fixture_root, &mut fixture_paths);

    let mut fixtures = fixture_paths
        .into_iter()
        .map(|path| {
            let relative_path = path
                .strip_prefix(&fixture_root)
                .expect("three.js fixture path should stay within the vendored fixture root");
            let normalized_path = relative_path.to_string_lossy().replace('\\', "/");
            let source = fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!("three.js fixture should be valid UTF-8: {normalized_path}")
            });

            FixtureSource {
                source_type: SourceType::from_path(&path)
                    .expect("three.js fixture path should infer JavaScript"),
                path: normalized_path,
                source,
            }
        })
        .collect::<Vec<_>>();

    fixtures.sort_by(|left, right| left.path.cmp(&right.path));
    assert_eq!(
        fixtures.len(),
        THREEJS_ESM_FILE_COUNT,
        "three.js ESM fixture set should include every vendored module",
    );

    fixtures
}

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
        SourceType::from_path(PARSE_SOURCE_FILENAME).expect("fixture path should infer JavaScript");
    let mut group = c.benchmark_group("parse");

    group.bench_function("oxc", |b| {
        b.iter(|| {
            black_box(parse_oxc_source(
                PARSE_SOURCE_FILENAME,
                black_box(PARSE_SOURCE),
                source_type,
            ));
        });
    });

    group.bench_function("swc", |b| {
        b.iter(|| {
            black_box(parse_swc_source(
                PARSE_SOURCE_FILENAME,
                black_box(PARSE_SOURCE),
            ));
        });
    });

    group.finish();
}

fn bench_threejs_esm_rayon(c: &mut Criterion) {
    let fixtures = load_threejs_esm_fixtures();
    let mut group = c.benchmark_group("threejs_esm_rayon");

    group.bench_function("oxc", |b| {
        b.iter(|| {
            let fixtures = black_box(fixtures.as_slice());
            black_box(
                fixtures
                    .par_iter()
                    .map(|fixture| {
                        parse_oxc_source(&fixture.path, &fixture.source, fixture.source_type)
                    })
                    .sum::<usize>(),
            );
        });
    });

    group.bench_function("swc", |b| {
        b.iter(|| {
            let fixtures = black_box(fixtures.as_slice());
            black_box(
                fixtures
                    .par_iter()
                    .map(|fixture| parse_swc_source(&fixture.path, &fixture.source))
                    .sum::<usize>(),
            );
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_repeat_string,
    bench_parsers,
    bench_threejs_esm_rayon
);
criterion_main!(benches);
