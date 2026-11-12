//! Shared settings and deterministic (fixed-seed) inputs for the benchmarks.
#![allow(dead_code)] // each bench file uses a subset

use criterion::Criterion;
use rand::rngs::Xoshiro256PlusPlus;
use rand::{RngExt, SeedableRng};
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

/// The benchmarks' random number generator. A named algorithm rather than `rand`'s `StdRng`, whose
/// algorithm may change between `rand` versions: the same seed must keep giving the same inputs.
pub type BenchRng = Xoshiro256PlusPlus;

pub fn rng(seed: u64) -> BenchRng {
    return BenchRng::seed_from_u64(seed);
}

/// Random ray tracer values, for any `rand` generator.
pub trait RandomInputs: RngExt {
    /// Components uniform in `[-1, 1)`.
    fn vector(&mut self) -> Vector {
        return Vector::new(
            self.random_range(-1.0..1.0),
            self.random_range(-1.0..1.0),
            self.random_range(-1.0..1.0),
        );
    }

    /// Uniformly distributed direction (rejection sampling in the unit ball).
    fn unit_vector(&mut self) -> Vector {
        loop {
            let vector = self.vector();
            let magnitude = vector.magnitude();
            if magnitude > 0.01 && magnitude <= 1.0 {
                return vector / magnitude;
            }
        }
    }

    /// Uniformly distributed inside the ball.
    fn point_in_ball(&mut self, center: Point, radius: f64) -> Point {
        let distance = radius * self.random_range(0.0..1.0_f64).cbrt();
        return center + self.unit_vector() * distance;
    }

    fn color(&mut self) -> Color {
        return Color::new(
            self.random_range(0.0..1.0),
            self.random_range(0.0..1.0),
            self.random_range(0.0..1.0),
        );
    }

    /// Random invertible transformation.
    fn transformation(&mut self) -> Transformation {
        let offset = self.vector() * 5.0;
        return transformations::translation(offset.x, offset.y, offset.z)
            * transformations::rotation_x(self.random_range(0.0..TAU))
            * transformations::rotation_y(self.random_range(0.0..TAU))
            * transformations::scaling(
                self.random_range(0.5..2.0),
                self.random_range(0.5..2.0),
                1.5,
            );
    }

    /// [`BATCH`] values from `generate`.
    fn batch<T>(&mut self, mut generate: impl FnMut(&mut Self) -> T) -> Vec<T>
    where
        Self: Sized,
    {
        return Vec::from_fn(BATCH, |_| generate(self));
    }
}

impl<R: RngExt + ?Sized> RandomInputs for R {}

/// Applies `operation` to every input into a reused output buffer (no allocation while timing).
pub fn map_into<I, O>(inputs: &[I], outputs: &mut [O], operation: impl Fn(&I) -> O) {
    for (output, input) in outputs.iter_mut().zip(black_box(inputs)) {
        *output = operation(input);
    }
    black_box(outputs);
}
