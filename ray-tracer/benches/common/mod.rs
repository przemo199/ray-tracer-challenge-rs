//! Shared settings and deterministic (fixed-seed) inputs for the benchmarks.
#![allow(dead_code)] // each bench file uses a subset

use criterion::Criterion;
use ray_tracer::primitives::{Color, Point, Transformation, Vector, transformations};
use std::f64::consts::TAU;
use std::hint::black_box;
use std::time::Duration;

/// Inputs per iteration; single operations are too fast (~1 ns) to time one at a time.
pub const BATCH: usize = 1024;

/// ~1.5 s per benchmark. Pinned runs of unchanged code differ by up to ~6%, hence the 5% threshold.
pub fn fast_config() -> Criterion {
    return Criterion::default()
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1))
        .sample_size(50)
        .noise_threshold(0.05);
}

/// SplitMix64 PRNG, so the benchmarks need no `rand` dependency.
pub struct Rng(u64);

impl Rng {
    pub const fn new(seed: u64) -> Self {
        return Self(seed);
    }

    /// Uniform value in `[min, max)`.
    pub fn range(&mut self, min: f64, max: f64) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        let unit = ((z ^ (z >> 31)) >> 11) as f64 / (1_u64 << 53) as f64;
        return (max - min).mul_add(unit, min);
    }

    pub fn vector(&mut self) -> Vector {
        return Vector::new(
            self.range(-1.0, 1.0),
            self.range(-1.0, 1.0),
            self.range(-1.0, 1.0),
        );
    }

    pub fn unit_vector(&mut self) -> Vector {
        loop {
            let vector = self.vector();
            let magnitude = vector.magnitude();
            if magnitude > 0.01 && magnitude <= 1.0 {
                return vector / magnitude;
            }
        }
    }

    pub fn point_in_ball(&mut self, center: Point, radius: f64) -> Point {
        return center + self.unit_vector() * (radius * self.range(0.0, 1.0).cbrt());
    }

    pub fn color(&mut self) -> Color {
        return Color::new(
            self.range(0.0, 1.0),
            self.range(0.0, 1.0),
            self.range(0.0, 1.0),
        );
    }

    /// Random invertible transformation.
    pub fn transformation(&mut self) -> Transformation {
        let offset = self.vector() * 5.0;
        return transformations::translation(offset.x, offset.y, offset.z)
            * transformations::rotation_x(self.range(0.0, TAU))
            * transformations::rotation_y(self.range(0.0, TAU))
            * transformations::scaling(self.range(0.5, 2.0), self.range(0.5, 2.0), 1.5);
    }

    pub fn batch<T>(&mut self, mut generate: impl FnMut(&mut Self) -> T) -> Vec<T> {
        return (0..BATCH).map(|_| generate(self)).collect();
    }
}

/// Applies `operation` to every input into a reused output buffer (no allocation while timing).
pub fn map_into<I, O>(inputs: &[I], outputs: &mut [O], operation: impl Fn(&I) -> O) {
    for (output, input) in outputs.iter_mut().zip(black_box(inputs)) {
        *output = operation(input);
    }
    black_box(outputs);
}
