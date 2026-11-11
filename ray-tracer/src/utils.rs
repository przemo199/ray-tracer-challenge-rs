use crate::consts::EPSILON;
use crate::primitives::Color;
use crate::primitives::transformations;
use crate::shapes::{Sphere, Transform as _};
use core::ops::Mul;

/// Trait for comparison using epsilon value
pub trait CoarseEq<Rhs = Self> {
    fn coarse_eq(&self, other: &Rhs) -> bool;

    fn coarse_ne(&self, other: &Rhs) -> bool {
        return !self.coarse_eq(other);
    }
}

impl CoarseEq for f64 {
    #[inline]
    fn coarse_eq(&self, other: &Self) -> bool {
        return self == other || (self - other).abs() < EPSILON;
    }
}

/// Trait for squaring values
pub trait Squared: Copy + Mul<Output = Self> {
    #[inline]
    fn squared(self) -> Self {
        return self * self;
    }
}

impl<T> Squared for T where T: Copy + Mul<Output = Self> {}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadratic_without_real_roots() {
        assert_eq!(solve_quadratic(1.0, 0.0, 1.0), None);
    }

    #[test]
    fn quadratic_with_one_root_returns_it_twice() {
        assert_eq!(solve_quadratic(1.0, -2.0, 1.0), Some((1.0, 1.0)));
    }

    #[test]
    fn quadratic_roots_are_ascending_for_positive_a() {
        // x² - x - 6 = (x + 2)(x - 3)
        assert_eq!(solve_quadratic(1.0, -1.0, -6.0), Some((-2.0, 3.0)));
    }

    #[test]
    fn coarse_eq_uses_epsilon() {
        assert!(1.0.coarse_eq(&(1.0 + EPSILON / 2.0)));
        assert!(1.0.coarse_ne(&(1.0 + EPSILON * 2.0)));
    }

    #[test]
    fn squared_and_cubed() {
        assert_eq!(3.0.squared(), 9.0);
        assert_eq!((-2.0_f64).cubed(), -8.0);
    }
}
