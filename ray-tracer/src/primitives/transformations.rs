use crate::primitives::{Matrix, Point, Vector};

pub type Transformation = Matrix<4>;

#[inline]
pub fn translation(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Transformation {
    let mut result = Transformation::IDENTITY;
    result.set_index(0, 3, x);
    result.set_index(1, 3, y);
    result.set_index(2, 3, z);
    return result;
}

#[inline]
pub fn scaling(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Transformation {
    let mut result = Transformation::IDENTITY;
    result.set_index(0, 0, x);
    result.set_index(1, 1, y);
    result.set_index(2, 2, z);
    return result;
}

#[inline]
pub fn rotation_x(theta: impl Into<f64>) -> Transformation {
    let theta = theta.into();
    let mut result = Transformation::IDENTITY;
    let cos = theta.cos();
    let sin = theta.sin();
    result.set_index(1, 1, cos);
    result.set_index(1, 2, -sin);
    result.set_index(2, 1, sin);
    result.set_index(2, 2, cos);
    return result;
}

#[inline]
pub fn rotation_y(theta: impl Into<f64>) -> Transformation {
    let theta = theta.into();
    let mut result = Transformation::IDENTITY;
    let cos = theta.cos();
    let sin = theta.sin();
    result.set_index(0, 0, cos);
    result.set_index(0, 2, sin);
    result.set_index(2, 0, -sin);
    result.set_index(2, 2, cos);
    return result;
}

#[inline]
pub fn rotation_z(theta: impl Into<f64>) -> Transformation {
    let theta = theta.into();
    let mut result = Transformation::IDENTITY;
    let cos = theta.cos();
    let sin = theta.sin();
    result.set_index(0, 0, cos);
    result.set_index(0, 1, -sin);
    result.set_index(1, 0, sin);
    result.set_index(1, 1, cos);
    return result;
}

#[inline]
pub fn shearing(
    xy: impl Into<f64>,
    xz: impl Into<f64>,
    yx: impl Into<f64>,
    yz: impl Into<f64>,
    zx: impl Into<f64>,
    zy: impl Into<f64>,
) -> Transformation {
    let mut result = Transformation::IDENTITY;
    result.set_index(0, 1, xy);
    result.set_index(0, 2, xz);
    result.set_index(1, 0, yx);
    result.set_index(1, 2, yz);
    result.set_index(2, 0, zx);
    result.set_index(2, 1, zy);
    return result;
}

#[inline]
pub fn view_transform(from: Point, to: Point, up: Vector) -> Transformation {
    let forward = (to - from).normalized();
    let up_normalized = up.normalized();
    let left_vector = forward.cross(&up_normalized);
    let true_up = left_vector.cross(&forward);
    let orientation = Transformation::new([
        [left_vector.x, left_vector.y, left_vector.z, 0.0],
        [true_up.x, true_up.y, true_up.z, 0.0],
        [-forward.x, -forward.y, -forward.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    return orientation * translation(-from.x, -from.y, -from.z);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::PI;

    #[test]
    fn point_translation() {
        let translation = translation(5, -3, 2);
        let point = Point::new(-3, 4, 5);
        assert_eq!(translation * point, Point::new(2, 1, 7));
    }

    #[test]
    fn point_translation_by_inverse() {
        let translation = translation(5, -3, 2).inverse();
        let point = Point::new(-3, 4, 5);
        assert_eq!(translation * point, Point::new(-8, 7, 3));
    }

    #[test]
    fn vector_translation() {
        let translation = translation(5, -3, 2);
        let vector = Vector::new(-3, 4, 5);
        assert_eq!(translation * vector, vector);
    }

    #[test]
    fn point_scaling() {
        let scaling = scaling(2, 3, 4);
        let point = Point::new(-4, 6, 8);
        assert_eq!(scaling * point, Point::new(-8, 18, 32));
    }

    #[test]
    fn vector_scaling() {
        let scaling = scaling(2, 3, 4);
        let vector = Vector::new(-4, 6, 8);
        assert_eq!(scaling * vector, Vector::new(-8, 18, 32));
    }

    #[test]
    fn vector_scaling_by_inverse() {
        let scaling = scaling(2, 3, 4).inverse();
        let vector = Vector::new(-4, 6, 8);
        assert_eq!(scaling * vector, Vector::new(-2, 2, 2));
    }

    #[test]
    fn point_reflection() {
        let reflection = scaling(-1, 1, 1);
        let point = Point::new(2, 3, 4);
        assert_eq!(reflection * point, Point::new(-2, 3, 4));
    }

    #[test]
    fn point_rotation_around_x() {
        let point = Point::new(0, 1, 0);
        let half_quarter_rotation = rotation_x(PI / 4.0);
        let full_quarter_rotation = rotation_x(PI / 2.0);
        assert_eq!(
            half_quarter_rotation * point,
            Point::new(0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0)
        );
        assert_eq!(full_quarter_rotation * point, Point::new(0, 0, 1));
    }

    #[test]
    fn point_rotation_inverse_around_x() {
        let point = Point::new(0, 1, 0);
        let full_quarter_rotation_inverse = rotation_x(PI / 4.0).inverse();
        assert_eq!(
            full_quarter_rotation_inverse * point,
            Point::new(0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0)
        );
    }

    #[test]
    fn point_rotation_around_y() {
        let point = Point::new(0, 0, 1);
        let half_quarter_rotation = rotation_y(PI / 4.0);
        let full_quarter_rotation = rotation_y(PI / 2.0);
        assert_eq!(
            half_quarter_rotation * point,
            Point::new(2.0_f64.sqrt() / 2.0, 0, 2.0_f64.sqrt() / 2.0)
        );
        assert_eq!(full_quarter_rotation * point, Point::new(1, 0, 0));
    }

    #[test]
    fn point_rotation_around_z() {
        let point = Point::new(0, 1, 0);
        let half_quarter_rotation = rotation_z(PI / 4.0);
        let full_quarter_rotation = rotation_z(PI / 2.0);
        assert_eq!(
            half_quarter_rotation * point,
            Point::new(-(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0, 0)
        );
        assert_eq!(full_quarter_rotation * point, Point::new(-1, 0, 0));
    }

    #[test]
    fn point_shearing() {
        let shearing_transformation = shearing(1, 0, 0, 0, 0, 0);
        let point = Point::new(2, 3, 4);
        assert_eq!(shearing_transformation * point, Point::new(5, 3, 4));

        let shearing_transformation = shearing(0, 1, 0, 0, 0, 0);
        let point = Point::new(2, 3, 4);
        assert_eq!(shearing_transformation * point, Point::new(6, 3, 4));

        let shearing_transformation = shearing(0, 0, 1, 0, 0, 0);
        let point = Point::new(2, 3, 4);
        assert_eq!(shearing_transformation * point, Point::new(2, 5, 4));

        let shearing_transformation = shearing(0, 0, 0, 1, 0, 0);
        let point = Point::new(2, 3, 4);
        assert_eq!(shearing_transformation * point, Point::new(2, 7, 4));

        let shearing_transformation = shearing(0, 0, 0, 0, 1, 0);
        let point = Point::new(2, 3, 4);
        assert_eq!(shearing_transformation * point, Point::new(2, 3, 6));

        let shearing_transformation = shearing(0, 0, 0, 0, 0, 1);
        let point = Point::new(2, 3, 4);
        assert_eq!(shearing_transformation * point, Point::new(2, 3, 7));
    }

    #[test]
    fn transformation_order() {
        let point_1 = Point::new(1, 0, 1);
        let rotation = rotation_x(PI / 2.0);
        let scaling = scaling(5, 5, 5);
        let translation = translation(10, 5, 7);
        let point_2 = rotation * point_1;
        assert_eq!(point_2, Point::new(1, -1, 0));
        let point_3 = scaling * point_2;
        assert_eq!(point_3, Point::new(5, -5, 0));
        let point_4 = translation * point_3;
        assert_eq!(point_4, Point::new(15, 0, 7));

        let total_transform = translation * scaling * rotation;
        let point_5 = total_transform * point_1;
        assert_eq!(point_5, Point::new(15, 0, 7));
    }

    #[test]
    fn transformation_matrix_for_default_orientation() {
        let from = Point::ORIGIN;
        let to = Point::new(0, 0, -1);
        let up = Vector::UP;
        let orientation = view_transform(from, to, up);
        assert_eq!(orientation, Transformation::IDENTITY);
    }

    #[test]
    fn transformation_matrix_with_positive_z_direction() {
        let from = Point::ORIGIN;
        let to = Point::new(0, 0, 1);
        let up = Vector::UP;
        let orientation = view_transform(from, to, up);
        assert_eq!(orientation, scaling(-1, 1, -1));
    }

    #[test]
    fn view_transformation_moves_world() {
        let from = Point::new(0, 0, 8);
        let to = Point::ORIGIN;
        let up = Vector::UP;
        let orientation = view_transform(from, to, up);
        assert_eq!(orientation, translation(0, 0, -8));
    }

    #[test]
    fn complex_view_transformation() {
        let from = Point::new(1, 3, 2);
        let to = Point::new(4, -2, 8);
        let up = Vector::new(1, 1, 0);
        let orientation = view_transform(from, to, up);
        let result = Transformation::new([
            [
                -0.5070925528371099,
                0.5070925528371099,
                0.6761234037828132,
                -2.366431913239846,
            ],
            [
                0.7677159338596801,
                0.6060915267313263,
                0.12121830534626524,
                -2.8284271247461894,
            ],
            [
                -0.35856858280031806,
                0.5976143046671968,
                -0.7171371656006361,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        assert_eq!(orientation, result);
    }
}
