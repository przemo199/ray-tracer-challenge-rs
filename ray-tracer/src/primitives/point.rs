use crate::primitives::Vector;
use crate::utils::CoarseEq;
use bincode::Encode;
use core::fmt::{Display, Formatter, Result};
use core::ops::{Add, Div, Index, Mul, Neg, Sub};

/// Struct representing point in three-dimensional space
#[derive(Clone, Copy, Debug, Encode)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub const W: f64 = 1.0;

    pub const ORIGIN: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Creates new instance of struct [Point]
    /// # Examples
    /// ```
    /// use ray_tracer::primitives::Point;
    ///
    /// let point = Point::new(1, 0.5, 0);
    ///
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 0.5);
    /// assert_eq!(point.z, 0.0);
    /// ```
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Self {
        return Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        };
    }

    pub fn from_fn(init: impl Fn(usize) -> f64) -> Self {
        return Point::new(init(0), init(1), init(2));
    }

    pub const fn values(&self) -> [f64; 4] {
        return [self.x, self.y, self.z, Self::W];
    }

    pub fn map(&self, f: impl Fn(f64) -> f64) -> Self {
        return Into::<[f64; 3]>::into(*self).map(f).into();
    }

    pub fn abs(&self) -> Self {
        return self.map(|value| value.abs());
    }
}

impl Default for Point {
    fn default() -> Self {
        return Self::ORIGIN;
    }
}

impl Display for Point {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        return formatter
            .debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish();
    }
}

impl PartialEq for Point {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        return std::ptr::eq(self, rhs)
            || self.x.coarse_eq(rhs.x) && self.y.coarse_eq(rhs.y) && self.z.coarse_eq(rhs.z);
    }
}

impl Add<Vector> for Point {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Vector) -> Self::Output {
        return Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z);
    }
}

impl Sub for Point {
    type Output = Vector;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        return Self::Output::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}

impl Sub<Vector> for Point {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Vector) -> Self::Output {
        return Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}

impl Mul<f64> for Point {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        return self.map(|value| value * rhs);
    }
}

impl Div<f64> for Point {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        return self.map(|value| value / rhs);
    }
}

impl Neg for Point {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        return self.map(f64::neg);
    }
}

impl Index<usize> for Point {
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

impl IntoIterator for Point {
    type Item = f64;
    type IntoIter = std::array::IntoIter<f64, 4>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        return IntoIterator::into_iter(self.values());
    }
}

impl<'a> IntoIterator for &'a Point {
    type Item = &'a f64;
    type IntoIter = std::array::IntoIter<&'a f64, 4>;

    fn into_iter(self) -> Self::IntoIter {
        return IntoIterator::into_iter([&self.x, &self.y, &self.z, &Point::W]);
    }
}

impl<T: Into<f64>> From<[T; 3]> for Point {
    fn from(value: [T; 3]) -> Self {
        let [x, y, z] = value;
        return Self::new(x, y, z);
    }
}

impl<T: Into<f64>> From<[T; 4]> for Point {
    fn from(value: [T; 4]) -> Self {
        let [x, y, z, ..] = value;
        return Self::new(x, y, z);
    }
}

impl From<Point> for [f64; 4] {
    fn from(value: Point) -> Self {
        return value.values();
    }
}

impl From<Point> for [f64; 3] {
    fn from(value: Point) -> Self {
        let [x, y, z, ..] = value.values();
        return [x, y, z];
    }
}

impl From<Vector> for Point {
    fn from(value: Vector) -> Self {
        return Point::new(value.x, value.y, value.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::EPSILON;

    #[test]
    fn new_point() {
        let point = Point::new(4, -4, 3);
        assert_eq!(point.x, 4.0);
        assert_eq!(point.y, -4.0);
        assert_eq!(point.z, 3.0);
    }

    #[test]
    fn eq_point() {
        let point_1 = Point::new(4, -4, 3);
        let point_2 = point_1;
        let point_3 = Point::new(4.1 + EPSILON, -4, 3);
        let point_4 = Point::new(4.0 + EPSILON - (EPSILON / 2.0), -4, 3);
        assert_eq!(point_1, point_2);
        assert_ne!(point_2, point_3);
        assert_eq!(point_2, point_4);
    }

    #[test]
    fn add_point_and_vector() {
        let point = Point::new(3, -2, 5);
        let vector = Vector::new(-2, 3, 1);
        assert_eq!(point + vector, Point::new(1, 1, 6));
    }

    #[test]
    fn subtract_vector_from_point() {
        let point = Point::new(3, 2, 1);
        let vector = Vector::new(5, 6, 7);
        assert_eq!(point - vector, Point::new(-2, -4, -6));
    }

    #[test]
    fn sub_point() {
        let point_1 = Point::new(3, 2, 1);
        let point_2 = Point::new(5, 6, 7);
        assert_eq!(point_1 - point_2, Vector::new(-2, -4, -6));
    }

    #[test]
    fn neg_point() {
        let point_1 = Point::new(1, -2, 3);
        let point_2 = Point::new(4, -4, 3);
        let point_3 = Point::new(4, -4, 3);
        assert_eq!(-point_1, Point::new(-1, 2, -3));
        assert_eq!(-point_2, Point::new(-4, 4, -3));
        assert_eq!(-point_3, Point::new(-4, 4, -3));
    }

    #[test]
    fn mul_point() {
        let point_1 = Point::new(1, -2, 3) * 3.5;
        let point_2 = Point::new(1, -2, 3) * 0.5;
        assert_eq!(point_1, Point::new(3.5, -7, 10.5));
        assert_eq!(point_2, Point::new(0.5, -1, 1.5));
    }

    #[test]
    fn div_point() {
        let point = Point::new(1, -2, 3) / 2.0;
        assert_eq!(point, Point::new(0.5, -1, 1.5));
    }
}
