use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::consts::EPSILON;

pub trait TupleTrait: Clone + Copy + Debug + PartialEq + Add<Self, Output=Self> + Sub<Self, Output=Self> + Mul<f64, Output=Self> + Div<f64, Output=Self> + Neg<Output=Self> {
    fn new(x: f64, y: f64, z: f64, w: f64) -> Self;
    fn is_vector(&self) -> bool;
    fn is_point(&self) -> bool;
    fn magnitude(&self) -> f64;
    fn normalize(&self) -> Self;
    fn dot(&self, rhs: &Self) -> f64;
    fn cross(&self, rhs: &Self) -> Self;
    fn reflect(&self, normal: &Self) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub struct Tuple {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Tuple {
    pub fn point(x: f64, y: f64, z: f64) -> Tuple {
        let w = 1.0;
        return Tuple { x, y, z, w };
    }

    pub fn vector(x: f64, y: f64, z: f64) -> Tuple {
        let w = 0.0;
        return Tuple { x, y, z, w };
    }

    pub fn get_values(&self) -> [f64; 4] {
        return [self.x, self.y, self.z, self.w];
    }
}

impl TupleTrait for Tuple {
    fn new(x: f64, y: f64, z: f64, w: f64) -> Tuple {
        return Tuple { x, y, z, w };
    }

    fn is_vector(&self) -> bool {
        return self.w == 0.0;
    }

    fn is_point(&self) -> bool {
        return self.w == 1.0;
    }

    fn magnitude(&self) -> f64 {
        if self.is_point() {
            panic!("Cannot calculate magnitude of a point");
        }
        return (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt();
    }

    fn normalize(&self) -> Tuple {
        if self.is_point() {
            panic!("Cannot normalize a point");
        }
        let magnitude = self.magnitude();
        return Tuple::new(self.x / magnitude, self.y / magnitude, self.z / magnitude, self.w / magnitude);
    }

    fn dot(&self, rhs: &Tuple) -> f64 {
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w;
    }

    fn cross(&self, rhs: &Tuple) -> Tuple {
        return Tuple::vector(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        );
    }

    fn reflect(&self, normal: &Tuple) -> Tuple {
        return *self - *normal * 2.0_f64 * self.dot(normal);
    }
}

impl PartialEq for Tuple {
    fn eq(&self, rhs: &Tuple) -> bool {
        return self.w == rhs.w &&
            (self.x - rhs.x).abs() < EPSILON &&
            (self.y - rhs.y).abs() < EPSILON &&
            (self.z - rhs.z).abs() < EPSILON;
    }
}

impl Add for Tuple {
    type Output = Tuple;

    fn add(self, rhs: Tuple) -> Self::Output {
        if self.is_point() && rhs.is_point() {
            panic!("Cannot add two points!");
        }
        return Tuple::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w);
    }
}

impl Sub for Tuple {
    type Output = Tuple;

    fn sub(self, rhs: Tuple) -> Self::Output {
        if self.is_vector() && rhs.is_point() {
            panic!("Cannot subtract point from a vector!");
        }
        return Tuple::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w);
    }
}

impl Neg for Tuple {
    type Output = Tuple;

    fn neg(self) -> Self::Output {
        return Tuple::new(-self.x, -self.y, -self.z, -self.w);
    }
}

impl Mul<f64> for Tuple {
    type Output = Tuple;

    fn mul(self, rhs: f64) -> Self::Output {
        return Tuple::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs);
    }
}

impl Div<f64> for Tuple {
    type Output = Tuple;

    fn div(self, rhs: f64) -> Self::Output {
        return Tuple::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_point() {
        let point = Tuple::point(4.0, -4.0, 3.0);
        assert_eq!(point.x, 4.0);
        assert_eq!(point.y, -4.0);
        assert_eq!(point.z, 3.0);
        assert_eq!(point.w, 1.0);
        assert!(!point.is_vector());
    }

    #[test]
    fn new_vector() {
        let vector = Tuple::vector(4.0, -4.0, 3.0);
        assert_eq!(vector.x, 4.0);
        assert_eq!(vector.y, -4.0);
        assert_eq!(vector.z, 3.0);
        assert_eq!(vector.w, 0.0);
        assert!(vector.is_vector());
    }

    #[test]
    fn eq_point() {
        let point1 = Tuple::point(4.0, -4.0, 3.0);
        let point2 = point1;
        let point3 = Tuple::point(4.1 + EPSILON, -4.0, 3.0);
        let point4 = Tuple::point(4.0 + EPSILON - (EPSILON / 2.0), -4.0, 3.0);
        assert_eq!(point1, point2);
        assert_ne!(point2, point3);
        assert_eq!(point2, point4);
    }

    #[test]
    fn eq_vector() {
        let vector1 = Tuple::vector(4.0, -4.0, 3.0);
        let vector2 = vector1;
        let vector3 = Tuple::vector(4.0 + EPSILON, -4.0, 3.0);
        let vector4 = Tuple::vector(4.0 + EPSILON - (EPSILON / 2.0), -4.0, 3.0);
        assert_eq!(vector1, vector2);
        assert_ne!(vector2, vector3);
        assert_eq!(vector2, vector4);
    }

    #[test]
    #[should_panic]
    fn add_points() {
        let point1 = Tuple::point(3.0, -2.0, 5.0);
        let point2 = Tuple::point(-2.0, 3.0, 1.0);
        let _ = point1 + point2;
    }

    #[test]
    fn add_vectors() {
        let vector1 = Tuple::vector(3.0, -2.0, 5.0);
        let vector2 = Tuple::vector(-2.0, 3.0, 1.0);
        let vector3 = vector1 + vector2;
        assert_eq!(vector3, Tuple::vector(1.0, 1.0, 6.0));
    }

    #[test]
    fn add_point_and_vector() {
        let point1 = Tuple::point(3.0, -2.0, 5.0);
        let point2 = Tuple::vector(-2.0, 3.0, 1.0);
        let point3 = point1 + point2;
        assert_eq!(point3, Tuple::point(1.0, 1.0, 6.0));
    }

    #[test]
    fn sub_points() {
        let point1 = Tuple::point(3.0, 2.0, 1.0);
        let point2 = Tuple::point(5.0, 6.0, 7.0);
        let point3 = point1 - point2;
        assert_eq!(point3, Tuple::vector(-2.0, -4.0, -6.0));
    }

    #[test]
    fn sub_vectors() {
        let vector1 = Tuple::vector(4.0, -4.0, 3.0);
        let vector2 = vector1;
        let vector3 = vector1 - vector2;
        assert_eq!(vector3, Tuple::vector(0.0, 0.0, 0.0));
    }

    #[test]
    #[should_panic]
    fn sub_vector_and_point() {
        let vector1 = Tuple::vector(4.0, -4.0, 3.0);
        let point1 = Tuple::point(4.0, -4.0, 3.0);
        let _ = vector1 - point1;
    }

    #[test]
    fn neg_tuple() {
        let tuple1 = Tuple::new(1.0, -2.0, 3.0, -4.0);
        let point1 = Tuple::point(4.0, -4.0, 3.0);
        let vector1 = Tuple::vector(4.0, -4.0, 3.0);
        assert_eq!(-point1, Tuple::new(-4.0, 4.0, -3.0, -1.0));
        assert_eq!(-vector1, Tuple::new(-4.0, 4.0, -3.0, 0.0));
        assert_eq!(-tuple1, Tuple::new(-1.0, 2.0, -3.0, 4.0));
    }

    #[test]
    fn mul_tuple() {
        let tuple1 = Tuple::new(1.0, -2.0, 3.0, -4.0) * 3.5;
        let tuple2 = Tuple::new(1.0, -2.0, 3.0, -4.0) * 0.5;
        assert_eq!(tuple1, Tuple::new(3.5, -7.0, 10.5, -14.0));
        assert_eq!(tuple2, Tuple::new(0.5, -1.0, 1.5, -2.0));
    }

    #[test]
    fn div_tuple() {
        let tuple1 = Tuple::new(1.0, -2.0, 3.0, -4.0) / 2.0;
        assert_eq!(tuple1, Tuple::new(0.5, -1.0, 1.5, -2.0));
    }

    #[test]
    #[should_panic]
    fn point_magnitude() {
        let point1 = Tuple::point(1.0, 2.0, 3.0);
        point1.magnitude();
    }

    #[test]
    fn vector_magnitude() {
        let vector1 = Tuple::vector(1.0, 0.0, 0.0);
        let vector2 = Tuple::vector(0.0, 1.0, 0.0);
        let vector3 = Tuple::vector(0.0, 0.0, 1.0);
        let vector4 = Tuple::vector(1.0, 2.0, 3.0);
        let vector5 = Tuple::vector(-1.0, -2.0, -3.0);
        assert_eq!(vector1.magnitude(), 1.0);
        assert_eq!(vector2.magnitude(), 1.0);
        assert_eq!(vector3.magnitude(), 1.0);
        assert_eq!(vector4.magnitude(), 14.0_f64.sqrt());
        assert_eq!(vector5.magnitude(), 14.0_f64.sqrt());
    }

    #[test]
    fn normalize_vector() {
        let vector1 = Tuple::vector(4.0, 0.0, 0.0);
        let vector2 = Tuple::vector(0.0, 4.0, 0.0);
        let vector3 = Tuple::vector(0.0, 0.0, 4.0);
        let vector4 = Tuple::vector(1.0, 2.0, 3.0);
        let normalised1 = vector1.normalize();
        let normalised2 = vector2.normalize();
        let normalised3 = vector3.normalize();
        let normalised4 = vector4.normalize();
        assert_eq!(normalised1, Tuple::vector(1.0, 0.0, 0.0));
        assert_eq!(normalised1.magnitude(), 1.0);
        assert_eq!(normalised2, Tuple::vector(0.0, 1.0, 0.0));
        assert_eq!(normalised2.magnitude(), 1.0);
        assert_eq!(normalised3, Tuple::vector(0.0, 0.0, 1.0));
        assert_eq!(normalised3.magnitude(), 1.0);
        let magnitude = vector4.magnitude();
        assert_eq!(normalised4, Tuple::vector(1.0 / magnitude, 2.0 / magnitude, 3.0 / magnitude));
        assert!(normalised4.magnitude() - 1.0 < EPSILON);
    }

    #[test]
    fn dot_product() {
        let vector1 = Tuple::vector(1.0, 2.0, 3.0);
        let vector2 = Tuple::vector(2.0, 3.0, 4.0);
        assert_eq!(vector1.dot(&vector2), 20.0);
    }

    #[test]
    fn cross_product() {
        let vector1 = Tuple::vector(1.0, 2.0, 3.0);
        let vector2 = Tuple::vector(2.0, 3.0, 4.0);
        assert_eq!(vector1.cross(&vector2), Tuple::vector(-1.0, 2.0, -1.0));
        assert_eq!(vector2.cross(&vector1), Tuple::vector(1.0, -2.0, 1.0));
    }

    #[test]
    fn reflect_vector() {
        let vector1 = Tuple::vector(1.0, -1.0, 0.0);
        let normal = Tuple::vector(0.0, 1.0, 0.0);
        let reflected = vector1.reflect(&normal);
        assert_eq!(reflected, Tuple::vector(1.0, 1.0, 0.0));
    }

    #[test]
    fn reflect_vector_on_slanted_surface() {
        let vector1 = Tuple::vector(0.0, -1.0, 0.0);
        let normal = Tuple::vector(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        let reflected = vector1.reflect(&normal);
        assert_eq!(reflected, Tuple::vector(1.0, 0.0, 0.0));
    }
}
