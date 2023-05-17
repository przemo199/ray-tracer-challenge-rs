use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};

use bincode::Encode;

use crate::utils::CloseEnough;

#[derive(Clone, Copy, Debug, Encode)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    /// Creates new instance of struct [Vector]
    /// # Examples
    /// ```
    ///     use raytracer::primitives::Vector;
    ///
    ///     let vector = Vector::new(1, 0.5, 0);
    ///
    ///     assert_eq!(vector.x, 1.0);
    ///     assert_eq!(vector.y, 0.5);
    ///     assert_eq!(vector.z, 0.0);
    /// ```
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Vector {
        return Vector { x: x.into(), y: y.into(), z: z.into() };
    }

    pub fn get_values(&self) -> [f64; 4] {
        return [self.x, self.y, self.z, 0.0];
    }

    pub fn magnitude(&self) -> f64 {
        return (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt();
    }

    pub fn normalized(&self) -> Vector {
        let magnitude = self.magnitude();
        return Vector::new(self.x / magnitude, self.y / magnitude, self.z / magnitude);
    }

    pub fn dot(&self, rhs: &Vector) -> f64 {
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }

    pub fn cross(&self, rhs: &Vector) -> Vector {
        return Vector::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        );
    }

    pub fn reflect(&self, normal: &Vector) -> Vector {
        return *self - *normal * 2.0_f64 * self.dot(normal);
    }
}

impl Default for Vector {
    fn default() -> Self {
        return Vector::new(0, 0, 0);
    }
}

impl Display for Vector {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        return formatter.debug_struct("Vector")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish();
    }
}

impl PartialEq for Vector {
    fn eq(&self, rhs: &Vector) -> bool {
        return self.x.close_enough(rhs.x) &&
            self.y.close_enough(rhs.y) &&
            self.z.close_enough(rhs.z);
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        return Vector::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z);
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        return Vector::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        return Vector::new(self.x * rhs, self.y * rhs, self.z * rhs);
    }
}

impl Div<f64> for Vector {
    type Output = Vector;

    fn div(self, rhs: f64) -> Self::Output {
        return Vector::new(self.x / rhs, self.y / rhs, self.z / rhs);
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        return Vector::new(-self.x, -self.y, -self.z);
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::consts::EPSILON;

    use super::*;

    #[test]
    fn new_vector() {
        let point = Vector::new(4, -4, 3);
        assert_eq!(point.x, 4.0);
        assert_eq!(point.y, -4.0);
        assert_eq!(point.z, 3.0);
    }

    #[test]
    fn eq_vector() {
        let vector_1 = Vector::new(4, -4, 3);
        let vector_2 = vector_1;
        let vector_3 = Vector::new(4.0 + EPSILON, -4, 3);
        let vector_4 = Vector::new(4.0 + EPSILON - (EPSILON / 2.0), -4, 3);
        assert_eq!(vector_1, vector_2);
        assert_ne!(vector_2, vector_3);
        assert_eq!(vector_2, vector_4);
    }

    #[test]
    fn add_vector() {
        let vector_1 = Vector::new(3, -2, 5);
        let vector_2 = Vector::new(-2, 3, 1);
        assert_eq!(vector_1 + vector_2, Vector::new(1, 1, 6));
    }

    #[test]
    fn sub_vector() {
        let vector_1 = Vector::new(4, -4, 3);
        let vector_2 = vector_1;
        assert_eq!(vector_1 - vector_2, Vector::new(0, 0, 0));
    }

    #[test]
    fn neg_vector() {
        let vector_1 = Vector::new(1, -2, 3);
        let vector_2 = Vector::new(4, -4, 3);
        let vector_3 = Vector::new(4, -4, 3);
        assert_eq!(-vector_1, Vector::new(-1, 2, -3));
        assert_eq!(-vector_2, Vector::new(-4, 4, -3));
        assert_eq!(-vector_3, Vector::new(-4, 4, -3));
    }

    #[test]
    fn mul_vector() {
        let vector_1 = Vector::new(1, -2, 3);
        let vector_2 = Vector::new(1, -2, 3);
        assert_eq!(vector_1 * 3.5, Vector::new(3.5, -7, 10.5));
        assert_eq!(vector_2 * 0.5, Vector::new(0.5, -1, 1.5));
    }

    #[test]
    fn div_vector() {
        let vector = Vector::new(1, -2, 3);
        assert_eq!(vector / 2.0, Vector::new(0.5, -1, 1.5));
    }

    #[rstest]
    #[case(Vector::new(1, 0, 0), 1.0)]
    #[case(Vector::new(0, 1, 0), 1.0)]
    #[case(Vector::new(0, 0, 1), 1.0)]
    #[case(Vector::new(1, 2, 3), 14.0_f64.sqrt())]
    #[case(Vector::new(-1, -2, -3), 14.0_f64.sqrt())]
    fn vector_magnitude(#[case] vector: Vector, #[case] magnitude: f64) {
        assert_eq!(vector.magnitude(), magnitude);
    }

    #[test]
    fn normalize_vector() {
        let vector1 = Vector::new(4, 0, 0);
        let vector2 = Vector::new(0, 4, 0);
        let vector3 = Vector::new(0, 0, 4);
        let vector4 = Vector::new(1, 2, 3);
        let normalised1 = vector1.normalized();
        let normalised2 = vector2.normalized();
        let normalised3 = vector3.normalized();
        let normalised4 = vector4.normalized();
        assert_eq!(normalised1, Vector::new(1, 0, 0));
        assert_eq!(normalised1.magnitude(), 1.0);
        assert_eq!(normalised2, Vector::new(0, 1, 0));
        assert_eq!(normalised2.magnitude(), 1.0);
        assert_eq!(normalised3, Vector::new(0, 0, 1));
        assert_eq!(normalised3.magnitude(), 1.0);
        assert_eq!(normalised4, Vector::new(0.2672612419124244, 0.5345224838248488, 0.8017837257372732));
    }

    #[test]
    fn magnitude_of_normalized_vector() {
        let vector = Vector::new(1, 2, 3);
        assert!(vector.normalized().magnitude().close_enough(1.0));
    }

    #[test]
    fn vector_dot_product() {
        let vector1 = Vector::new(1, 2, 3);
        let vector2 = Vector::new(2, 3, 4);
        assert_eq!(vector1.dot(&vector2), 20.0);
    }

    #[test]
    fn cross_product() {
        let vector1 = Vector::new(1, 2, 3);
        let vector2 = Vector::new(2, 3, 4);
        assert_eq!(vector1.cross(&vector2), Vector::new(-1, 2, -1));
        assert_eq!(vector2.cross(&vector1), Vector::new(1, -2, 1));
    }

    #[test]
    fn reflect_vector() {
        let vector = Vector::new(1, -1, 0);
        let normal = Vector::new(0, 1, 0);
        let reflected = vector.reflect(&normal);
        assert_eq!(reflected, Vector::new(1, 1, 0));
    }

    #[test]
    fn reflect_vector_on_slanted_surface() {
        let vector = Vector::new(0, -1, 0);
        let normal = Vector::new(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0);
        let reflected = vector.reflect(&normal);
        assert_eq!(reflected, Vector::new(1, 0, 0));
    }
}
