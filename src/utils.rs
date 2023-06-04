use std::ops::Mul;
use crate::consts::EPSILON;
use crate::primitives::Color;
use crate::primitives::transformations;
use crate::shapes::Sphere;

/// Trait for imprecise comparison between floats
pub trait CoarseEq where Self: Sized {
    const EPSILON: Self;

    fn coarse_eq(&self, rhs: Self) -> bool;
}

impl CoarseEq for f32 {
    const EPSILON: f32 = EPSILON as f32;

    #[inline(always)]
    fn coarse_eq(&self, rhs: Self) -> bool {
        return (self - rhs).abs() < CoarseEq::EPSILON;
    }
}

impl CoarseEq for f64 {
    const EPSILON: f64 = EPSILON;

    #[inline(always)]
    fn coarse_eq(&self, rhs: Self) -> bool {
        return (self - rhs).abs() < CoarseEq::EPSILON;
    }
}

/// Trait for efficiently squaring value
pub trait Squared: Copy + Mul<Self, Output=Self> {
    #[inline(always)]
    fn squared(self) -> Self {
        return self * self;
    }
}

impl<T> Squared for T where T: Copy + Mul<Self, Output=Self> {}

#[inline(always)]
pub fn solve_quadratic(a: f64, b: f64, c: f64) -> Option<(f64, f64)> {
    let discriminant = b.squared() - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let double_a = 2.0 * a;
    let discriminant_root = discriminant.sqrt();
    let solution_1 = (-b - discriminant_root) / double_a;
    let solution_2 = (-b + discriminant_root) / double_a;
    return Some((solution_1, solution_2));
}

#[inline(always)]
pub fn any_as_u8_slice<T: Sized>(value: &T) -> &[u8] {
    return unsafe {
        core::slice::from_raw_parts(
            (value as *const T) as *const u8,
            core::mem::size_of::<T>(),
        )
    };
}

pub fn world_default_sphere_1() -> Sphere {
    let mut sphere = Sphere::default();
    sphere.material.color = Color::new(0.8, 1, 0.6);
    sphere.material.diffuse = 0.7;
    sphere.material.specular = 0.2;
    return sphere;
}

pub fn world_default_sphere_2() -> Sphere {
    let mut sphere = Sphere::default();
    sphere.transformation = transformations::scaling(0.5, 0.5, 0.5);
    return sphere;
}
