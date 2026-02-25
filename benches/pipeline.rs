//! Benchmarks for the px pipeline.

use std::fs;
use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use px::parser::{parse_palette, parse_shape_file};
use px::render::{DitherMethod, P8Config};
use px::types::{BuiltinStamps, Colour};
use px::{quantize_sheet, RenderedShape, ShapeRenderer, SheetPacker};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn load_fixture(name: &str) -> String {
    fs::read_to_string(fixtures_dir().join(name)).unwrap()
}

// -- Parsing benchmarks --

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    let shape_source = load_fixture("sprites.shape.md");
    let palette_source = load_fixture("test.palette.md");

    // Small shape: parse a single shape definition
    let small_shape = "---\nname: tiny\n---\n\n```px\n##\n##\n```\n";

    group.bench_function("parse_shape_small", |b| {
        b.iter(|| parse_shape_file(black_box(small_shape)).unwrap())
    });

    group.bench_function("parse_shape_multi", |b| {
        b.iter(|| parse_shape_file(black_box(&shape_source)).unwrap())
    });

    group.bench_function("parse_palette", |b| {
        b.iter(|| parse_palette(black_box(&palette_source)).unwrap())
    });

    group.finish();
}

// -- Rendering benchmarks --

fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    let palette_source = load_fixture("test.palette.md");
    let builders = parse_palette(&palette_source).unwrap();
    let palette = builders.into_iter().next().unwrap().build(None).unwrap();
    let builtins = BuiltinStamps::all();

    // Small shape: 4x4
    let small_shapes = parse_shape_file(
        "---\nname: small\n---\n\n```px\n+--+\n|..|\n|..|\n+--+\n```\n",
    )
    .unwrap();
    let small = &small_shapes[0];

    // Medium shape: 16x16
    let mut grid_16 = String::from("---\nname: medium\n---\n\n```px\n");
    for row in 0..16 {
        for col in 0..16 {
            if row == 0 || row == 15 || col == 0 || col == 15 {
                grid_16.push('#');
            } else {
                grid_16.push('.');
            }
        }
        grid_16.push('\n');
    }
    grid_16.push_str("```\n");
    let medium_shapes = parse_shape_file(&grid_16).unwrap();
    let medium = &medium_shapes[0];

    group.bench_function("render_shape_small", |b| {
        b.iter(|| {
            let mut renderer = ShapeRenderer::new(&palette);
            renderer.add_stamps(builtins.iter());
            renderer.render(black_box(small))
        })
    });

    group.bench_function("render_shape_medium", |b| {
        b.iter(|| {
            let mut renderer = ShapeRenderer::new(&palette);
            renderer.add_stamps(builtins.iter());
            renderer.render(black_box(medium))
        })
    });

    group.finish();
}

// -- Packing benchmarks --

fn bench_packing(c: &mut Criterion) {
    let mut group = c.benchmark_group("packing");

    let palette_source = load_fixture("test.palette.md");
    let builders = parse_palette(&palette_source).unwrap();
    let palette = builders.into_iter().next().unwrap().build(None).unwrap();
    let builtins = BuiltinStamps::all();

    let shape_source = load_fixture("sprites.shape.md");
    let shapes = parse_shape_file(&shape_source).unwrap();

    let mut renderer = ShapeRenderer::new(&palette);
    renderer.add_stamps(builtins.iter());

    let rendered_small: Vec<RenderedShape> = shapes.iter().map(|s| renderer.render(s)).collect();

    // Generate 10+ sprites for medium bench
    let mut rendered_medium: Vec<RenderedShape> = Vec::new();
    for i in 0..12 {
        let mut r = rendered_small[i % rendered_small.len()].clone();
        r.name = format!("sprite-{}", i);
        rendered_medium.push(r);
    }

    let packer = SheetPacker::new(0);

    group.bench_function("pack_sheet_small", |b| {
        b.iter(|| packer.pack(black_box(&rendered_small)))
    });

    group.bench_function("pack_sheet_medium", |b| {
        b.iter(|| packer.pack(black_box(&rendered_medium)))
    });

    group.finish();
}

// -- Quantization benchmarks --

fn bench_quantization(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantization");

    // Generate a 128x128 pixel grid with varied colours
    let pixels: Vec<Vec<Colour>> = (0..128)
        .map(|y| {
            (0..128)
                .map(|x| {
                    Colour::rgb(
                        ((x * 2) % 256) as u8,
                        ((y * 2) % 256) as u8,
                        (((x + y) * 3) % 256) as u8,
                    )
                })
                .collect()
        })
        .collect();

    let config_none = P8Config {
        dither: DitherMethod::None,
        transparent_index: 0,
    };

    let config_ordered = P8Config {
        dither: DitherMethod::Ordered,
        transparent_index: 0,
    };

    let config_fs = P8Config {
        dither: DitherMethod::FloydSteinberg,
        transparent_index: 0,
    };

    group.bench_function("quantize_direct", |b| {
        b.iter(|| quantize_sheet(black_box(&pixels), &config_none))
    });

    group.bench_function("dither_ordered", |b| {
        b.iter(|| quantize_sheet(black_box(&pixels), &config_ordered))
    });

    group.bench_function("dither_floyd_steinberg", |b| {
        b.iter(|| quantize_sheet(black_box(&pixels), &config_fs))
    });

    group.finish();
}

criterion_group!(benches, bench_parsing, bench_rendering, bench_packing, bench_quantization);
criterion_main!(benches);
