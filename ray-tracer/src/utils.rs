use crate::consts::EPSILON;
use crate::primitives::Color;
use crate::primitives::transformations;
use crate::shapes::{Sphere, Transform};
use core::ops::Mul;

/// Trait for comparison using epsilon value
pub trait CoarseEq<Rhs = Self> {
    fn coarse_eq(&self, rhs: &Rhs) -> bool;

    fn coarse_ne(&self, rhs: &Rhs) -> bool {
        return !self.coarse_eq(rhs);
    }
}

impl CoarseEq for f64 {
    #[inline]
    fn coarse_eq(&self, rhs: &Self) -> bool {
        if self == rhs {
            return true;
        }
        return (self - rhs).abs() < EPSILON;
    }
}

/// Trait for squaring values
pub trait Squared: Copy + Mul<Self, Output = Self> {
    #[inline]
    fn squared(self) -> Self {
        return self * self;
    }
}

impl<T> Squared for T where T: Copy + Mul<Self, Output = Self> {}

/// Trait for cubing values
pub trait Cubed: Squared {
    #[inline]
    fn cubed(self) -> Self {
        return self.squared() * self;
    }
}

impl<T> Cubed for T where T: Squared {}

#[inline]
pub fn solve_quadratic(a: f64, b: f64, c: f64) -> Option<(f64, f64)> {
    let discriminant = (4.0 * a).mul_add(-c, b.squared());
    if discriminant < 0.0 {
        return None;
    }
    let double_a = 2.0 * a;
    let discriminant_root = discriminant.sqrt();
    let solution_1 = (-b - discriminant_root) / double_a;
    let solution_2 = (-b + discriminant_root) / double_a;
    return Some((solution_1, solution_2));
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
    sphere.set_transformation(transformations::scaling(0.5, 0.5, 0.5));
    return sphere;
}
