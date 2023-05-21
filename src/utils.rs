use std::ops::Mul;
use crate::consts::EPSILON;
use crate::primitives::Color;
use crate::primitives::transformations;
use crate::shapes::Sphere;

/// Trait for imprecise comparison between floats
pub trait CloseEnough where Self: Sized {
    const EPSILON: Self;

    fn close_enough(&self, rhs: Self) -> bool;
}

impl CloseEnough for f32 {
    const EPSILON: f32 = EPSILON as f32;

    #[inline(always)]
    fn close_enough(&self, rhs: Self) -> bool {
        return (self - rhs).abs() < CloseEnough::EPSILON;
    }
}

impl CloseEnough for f64 {
    const EPSILON: f64 = EPSILON;

    #[inline(always)]
    fn close_enough(&self, rhs: Self) -> bool {
        return (self - rhs).abs() < CloseEnough::EPSILON;
    }
}

/// Trait for efficiently squaring value
pub trait Squared: Copy + Mul<Self, Output=Self> {
    #[inline(always)]
    fn squared(self) -> Self {
        return self * self;
    }
}

impl Squared for f32 {}

impl Squared for f64 {}

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
