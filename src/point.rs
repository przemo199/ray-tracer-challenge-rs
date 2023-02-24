use std::ops::{Add, Div, Mul, Neg, Sub};
use crate::utils::CloseEnough;
use crate::vector::Vector;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Point {
        return Point { x, y, z };
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
        return Point::new(0.0, 0.0, 0.0)
    }
}

impl PartialEq for Point {
    fn eq(&self, rhs: &Point) -> bool {
        return self.x.close_enough(rhs.x) &&
            self.y.close_enough(rhs.y) &&
            self.z.close_enough(rhs.z);
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
    use super::*;
    use crate::consts::EPSILON;

    #[test]
    fn new_point() {
        let point = Point::new(4.0, -4.0, 3.0);
        assert_eq!(point.x, 4.0);
        assert_eq!(point.y, -4.0);
        assert_eq!(point.z, 3.0);
    }

    #[test]
    fn eq_point() {
        let point1 = Point::new(4.0, -4.0, 3.0);
        let point2 = point1;
        let point3 = Point::new(4.1 + EPSILON, -4.0, 3.0);
        let point4 = Point::new(4.0 + EPSILON - (EPSILON / 2.0), -4.0, 3.0);
        assert_eq!(point1, point2);
        assert_ne!(point2, point3);
        assert_eq!(point2, point4);
    }

    #[test]
    fn add_point_and_vector() {
        let point1 = Point::new(3.0, -2.0, 5.0);
        let vector = Vector::new(-2.0, 3.0, 1.0);
        let point2 = point1 + vector;
        assert_eq!(point2, Point::new(1.0, 1.0, 6.0));
    }

    #[test]
    fn sub_point() {
        let point1 = Point::new(3.0, 2.0, 1.0);
        let point2 = Point::new(5.0, 6.0, 7.0);
        let vector = point1 - point2;
        assert_eq!(vector, Vector::new(-2.0, -4.0, -6.0));
    }

    #[test]
    fn neg_point() {
        let point1 = Point::new(1.0, -2.0, 3.0);
        let point2 = Point::new(4.0, -4.0, 3.0);
        let point3 = Point::new(4.0, -4.0, 3.0);
        assert_eq!(-point1, Point::new(-1.0, 2.0, -3.0));
        assert_eq!(-point2, Point::new(-4.0, 4.0, -3.0));
        assert_eq!(-point3, Point::new(-4.0, 4.0, -3.0));
    }

    #[test]
    fn mul_point() {
        let point1 = Point::new(1.0, -2.0, 3.0) * 3.5;
        let point2 = Point::new(1.0, -2.0, 3.0) * 0.5;
        assert_eq!(point1, Point::new(3.5, -7.0, 10.5));
        assert_eq!(point2, Point::new(0.5, -1.0, 1.5));
    }

    #[test]
    fn div_point() {
        let point = Point::new(1.0, -2.0, 3.0) / 2.0;
        assert_eq!(point, Point::new(0.5, -1.0, 1.5));
    }
}
