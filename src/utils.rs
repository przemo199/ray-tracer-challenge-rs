use std::ops::Sub;
use crate::color::Color;
use crate::consts::EPSILON;
use crate::sphere::Sphere;
use crate::transformations::Transformations;

pub trait CloseEnough {
    #[inline(always)]
    fn close_enough(&self, rhs: Self) -> bool where Self: Copy + Sub<Self, Output=f64> {
        return (*self - rhs).abs() < EPSILON;
    }
}

impl CloseEnough for f64 {}

pub fn world_default_sphere_1() -> Sphere {
    let mut sphere = Sphere::default();
    sphere.material.color = Color::new(0.8, 1.0, 0.6);
    sphere.material.diffuse = 0.7;
    sphere.material.specular = 0.2;
    return sphere;
}

pub fn world_default_sphere_2() -> Sphere {
    let mut sphere = Sphere::default();
    sphere.transformation = Transformations::scaling(0.5, 0.5, 0.5);
    return sphere;
}
