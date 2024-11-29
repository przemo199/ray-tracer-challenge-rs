use crate::primitives::Point;
use crate::utils::{CoarseEq, Squared};
use bincode::Encode;
use core::fmt::{Display, Formatter, Result};
use core::ops::{Add, Div, Mul, Neg, Sub};
use std::ops::Index;

#[derive(Clone, Copy, Debug, Encode)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub const W: f64 = 0.0;

    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const UP: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    pub const DOWN: Self = Self {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    };

    pub const RIGHT: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    pub const LEFT: Self = Self {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    };

    pub const FORWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub const BACKWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    /// Creates new instance of struct [Vector]
    /// # Examples
    /// ```
    /// use ray_tracer::primitives::Vector;
    /// let vector = Vector::new(1, 0.5, 0);
    /// assert_eq!(vector.x, 1.0);
    /// assert_eq!(vector.y, 0.5);
    /// assert_eq!(vector.z, 0.0);
    /// ```
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Self {
        return Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        };
    }

    pub fn from_fn(init: impl Fn(usize) -> f64) -> Self {
        return Vector::new(init(0), init(1), init(2));
    }

    pub const fn values(&self) -> [f64; 4] {
        return [self.x, self.y, self.z, Self::W];
    }

    #[inline]
    pub fn magnitude(&self) -> f64 {
        return (self.x.squared() + self.y.squared() + self.z.squared()).sqrt();
    }

    pub fn normalized(&self) -> Self {
        let magnitude = self.magnitude();
        return self.map(|value| value / magnitude);
    }

    pub fn dot(&self, rhs: &Self) -> f64 {
        return self.z.mul_add(rhs.z, self.x.mul_add(rhs.x, self.y * rhs.y));
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        return Self::new(
            self.y.mul_add(rhs.z, -self.z * rhs.y),
            self.z.mul_add(rhs.x, -self.x * rhs.z),
            self.x.mul_add(rhs.y, -self.y * rhs.x),
        );
    }

    pub fn reflect(&self, normal: &Self) -> Self {
        return *self - (*normal * 2.0_f64 * self.dot(normal));
    }

    pub fn map(&self, f: impl Fn(f64) -> f64) -> Self {
        return Into::<[f64; 3]>::into(*self).map(f).into();
    }

    pub fn abs(&self) -> Self {
        return self.map(|value| value.abs());
    }
}

impl Default for Vector {
    fn default() -> Self {
        return Self::ZERO;
    }
}

impl Display for Vector {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        return formatter
            .debug_struct("Vector")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish();
    }
}

impl PartialEq for Vector {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs)
            || self.x.coarse_eq(rhs.x) && self.y.coarse_eq(rhs.y) && self.z.coarse_eq(rhs.z);
    }
}

impl Add for Vector {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        return Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z);
    }
}

impl Sub for Vector {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        return Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        return self.map(|value| value * rhs);
    }
}

impl Div<f64> for Vector {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        return self.map(|value| value / rhs);
    }
}

impl Neg for Vector {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        return self.map(|value| -value);
    }
}

impl Index<usize> for Vector {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        return match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &Self::W,
            _ => panic!(
                "index out of bounds: the len is 4 but the index is {}",
                index
            ),
        };
    }
}

impl IntoIterator for Vector {
    type Item = f64;
    type IntoIter = std::array::IntoIter<f64, 4>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        return IntoIterator::into_iter(self.values());
    }
}

impl<'a> IntoIterator for &'a Vector {
    type Item = &'a f64;
    type IntoIter = std::array::IntoIter<&'a f64, 4>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        return IntoIterator::into_iter([&self.x, &self.y, &self.z, &Vector::W]);
    }
}

impl<T: Into<f64>> From<[T; 3]> for Vector {
    fn from(value: [T; 3]) -> Self {
        let [x, y, z] = value;
        return Self::new(x, y, z);
    }
}

impl<T: Into<f64>> From<[T; 4]> for Vector {
    fn from(value: [T; 4]) -> Self {
        let [x, y, z, ..] = value;
        return Self::new(x, y, z);
    }
}

impl From<Vector> for [f64; 4] {
    fn from(value: Vector) -> Self {
        return value.values();
    }
}

impl From<Vector> for [f64; 3] {
    fn from(value: Vector) -> Self {
        let [x, y, z, ..] = value.values();
        return [x, y, z];
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        return Vector::new(value.x, value.y, value.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::EPSILON;
    use rstest::rstest;

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
        assert_eq!(vector_1 - vector_2, Vector::ZERO);
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
        assert_eq!(vector_1 * 3.5, Vector::new(3.5, -7, 10.5));
        assert_eq!(vector_1 * 0.5, Vector::new(0.5, -1, 1.5));
    }

    #[test]
    fn div_vector() {
        let vector = Vector::new(1, -2, 3);
        assert_eq!(vector / 2.0, Vector::new(0.5, -1, 1.5));
    }

    #[rstest]
    #[case(Vector::RIGHT, 1.0)]
    #[case(Vector::UP, 1.0)]
    #[case(Vector::FORWARD, 1.0)]
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
        assert_eq!(normalised1, Vector::RIGHT);
        assert_eq!(normalised1.magnitude(), 1.0);
        assert_eq!(normalised2, Vector::UP);
        assert_eq!(normalised2.magnitude(), 1.0);
        assert_eq!(normalised3, Vector::FORWARD);
        assert_eq!(normalised3.magnitude(), 1.0);
        assert_eq!(
            normalised4,
            Vector::new(0.2672612419124244, 0.5345224838248488, 0.8017837257372732)
        );
    }

    #[test]
    fn magnitude_of_normalized_vector() {
        let vector = Vector::new(1, 2, 3);
        assert_eq!(vector.normalized().magnitude(), 1.0);
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
        let normal = Vector::UP;
        let reflected = vector.reflect(&normal);
        assert_eq!(reflected, Vector::new(1, 1, 0));
    }

    #[test]
    fn reflect_vector_on_slanted_surface() {
        let vector = Vector::DOWN;
        let normal = Vector::new(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0);
        let reflected = vector.reflect(&normal);
        assert_eq!(reflected, Vector::RIGHT);
    }
}
