use crate::primitives::Vector;
use crate::utils::CoarseEq;
use bincode::Encode;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Struct representing point in three dimensional space
#[derive(Clone, Copy, Debug, Encode)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub const ORIGIN: Point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Creates new instance of struct [Point]
    /// # Examples
    /// ```
    /// use raytracer::primitives::Point;
    ///
    /// let point = Point::new(1, 0.5, 0);
    ///
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 0.5);
    /// assert_eq!(point.z, 0.0);
    /// ```
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Point {
        return Point {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        };
    }

    pub fn get_values(&self) -> [f64; 4] {
        return [self.x, self.y, self.z, 1.0];
    }

    fn cross(&self, rhs: &Point) -> Point {
        return Point::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        );
    }
}

impl Default for Point {
    fn default() -> Self {
        return Point::ORIGIN;
    }
}

impl Display for Point {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        return formatter
            .debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish();
    }
}

impl PartialEq for Point {
    fn eq(&self, rhs: &Point) -> bool {
        return self.x.coarse_eq(rhs.x) && self.y.coarse_eq(rhs.y) && self.z.coarse_eq(rhs.z);
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Self::Output {
        return Point::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z);
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Self::Output {
        return Vector::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector) -> Self::Output {
        return Point::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z);
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Self::Output {
        return Point::new(self.x * rhs, self.y * rhs, self.z * rhs);
    }
}

impl Div<f64> for Point {
    type Output = Point;

    fn div(self, rhs: f64) -> Self::Output {
        return Point::new(self.x / rhs, self.y / rhs, self.z / rhs);
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        return Point::new(-self.x, -self.y, -self.z);
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::EPSILON;

    use super::*;

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
