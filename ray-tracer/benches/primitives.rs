//! Tier 1: single math operations on `Vector`, `Color` and `Matrix<4>`.
//!
//! Run: `cargo bench -p ray-tracer --bench primitives [-- <filter>]`

mod common;

use common::{BATCH, RandomInputs, fast_config, map_into, rng};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use rand::RngExt;
use ray_tracer::primitives::{Color, Point, Transformation, Vector, transformations};

fn bench_vector(criterion: &mut Criterion) {
    let mut rng = rng(1);
    let pairs = rng.batch(|rng| (rng.vector(), rng.unit_vector()));
    let scalars = rng.batch(|rng| (rng.vector(), rng.random_range(-2.0..2.0)));
    let mut vectors = vec![Vector::ZERO; BATCH];
    let mut floats = vec![0.0; BATCH];

    let mut group = criterion.benchmark_group("vector");
    group.throughput(Throughput::Elements(BATCH as u64));
    group.bench_function("add", |bencher| {
        bencher.iter(|| map_into(&pairs, &mut vectors, |&(a, b)| a + b));
    });
    group.bench_function("scale", |bencher| {
        bencher.iter(|| map_into(&scalars, &mut vectors, |&(a, s)| a * s));
    });
    group.bench_function("dot", |bencher| {
        bencher.iter(|| map_into(&pairs, &mut floats, |(a, b)| a.dot(b)));
    });
    group.bench_function("cross", |bencher| {
        bencher.iter(|| map_into(&pairs, &mut vectors, |(a, b)| a.cross(b)));
    });
    group.bench_function("normalized", |bencher| {
        bencher.iter(|| map_into(&pairs, &mut vectors, |(a, _)| a.normalized()));
    });
    group.bench_function("reflect", |bencher| {
        bencher.iter(|| map_into(&pairs, &mut vectors, |(a, normal)| a.reflect(normal)));
    });
    group.finish();
}

fn bench_color(criterion: &mut Criterion) {
    let mut rng = rng(2);
    let pairs = rng.batch(|rng| (rng.color(), rng.color()));
    let scalars = rng.batch(|rng| (rng.color(), rng.random_range(0.0..1.0)));
    let mut colors = vec![Color::BLACK; BATCH];

    let mut group = criterion.benchmark_group("color");
    group.throughput(Throughput::Elements(BATCH as u64));
    group.bench_function("add", |bencher| {
        bencher.iter(|| map_into(&pairs, &mut colors, |&(a, b)| a + b));
    });
    group.bench_function("mul", |bencher| {
        bencher.iter(|| map_into(&pairs, &mut colors, |&(a, b)| a * b));
    });
    group.bench_function("scale", |bencher| {
        bencher.iter(|| map_into(&scalars, &mut colors, |&(a, s)| a * s));
    });
    group.finish();
}

fn bench_matrix(criterion: &mut Criterion) {
    let mut rng = rng(3);
    let points = rng.batch(|rng| (rng.transformation(), Point::from(rng.vector())));
    let vectors = rng.batch(|rng| (rng.transformation(), rng.vector()));
    let matrices = rng.batch(|rng| (rng.transformation(), rng.transformation()));
    let views = rng.batch(|rng| {
        let from = Point::from(rng.vector() * 5.0);
        let to = rng.point_in_ball(Point::ORIGIN, 1.0);
        return (from, to, Vector::UP);
    });
    let mut out_points = vec![Point::ORIGIN; BATCH];
    let mut out_vectors = vec![Vector::ZERO; BATCH];
    let mut out_matrices = vec![Transformation::IDENTITY; BATCH];

    let mut group = criterion.benchmark_group("matrix4");
    group.throughput(Throughput::Elements(BATCH as u64));
    group.bench_function("mul_point", |bencher| {
        bencher.iter(|| map_into(&points, &mut out_points, |&(m, p)| m * p));
    });
    group.bench_function("mul_vector", |bencher| {
        bencher.iter(|| map_into(&vectors, &mut out_vectors, |&(m, v)| m * v));
    });
    group.bench_function("mul_matrix", |bencher| {
        bencher.iter(|| map_into(&matrices, &mut out_matrices, |&(a, b)| a * b));
    });
    group.bench_function("transpose", |bencher| {
        bencher.iter(|| map_into(&matrices, &mut out_matrices, |(a, _)| a.transpose()));
    });
    group.bench_function("inverse", |bencher| {
        bencher.iter(|| map_into(&matrices, &mut out_matrices, |(a, _)| a.inverse()));
    });
    group.bench_function("view_transform", |bencher| {
        bencher.iter(|| {
            map_into(&views, &mut out_matrices, |&(from, to, up)| {
                transformations::view_transform(from, to, up)
            });
        });
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = fast_config();
    targets = bench_vector, bench_color, bench_matrix
}
criterion_main!(benches);
