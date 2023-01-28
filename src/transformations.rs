use crate::{Matrix, Tuple, TupleTrait};

pub struct Transformations {}

impl Transformations {
    pub fn new(content: Vec<f64>) -> Matrix {
        return Matrix::new(content);
    }

    pub fn identity() -> Matrix {
        return Matrix::identity();
    }

    pub fn translation(x: f64, y: f64, z: f64) -> Matrix {
        let mut result = Transformations::identity();
        result.set_index(0, 3, x);
        result.set_index(1, 3, y);
        result.set_index(2, 3, z);
        return result;
    }

    pub fn scaling(x: f64, y: f64, z: f64) -> Matrix {
        let mut result = Transformations::identity();
        result.set_index(0, 0, x);
        result.set_index(1, 1, y);
        result.set_index(2, 2, z);
        return result;
    }

    pub fn rotation_x(theta: f64) -> Matrix {
        let mut result = Matrix::identity();
        let cos = theta.cos();
        let sin = theta.sin();
        result.set_index(1, 1, cos);
        result.set_index(1, 2, -sin);
        result.set_index(2, 1, sin);
        result.set_index(2, 2, cos);
        return result;
    }

    pub fn rotation_y(theta: f64) -> Matrix {
        let mut result = Matrix::identity();
        let cos = theta.cos();
        let sin = theta.sin();
        result.set_index(0, 0, cos);
        result.set_index(0, 2, sin);
        result.set_index(2, 0, -sin);
        result.set_index(2, 2, cos);
        return result;
    }

    pub fn rotation_z(theta: f64) -> Matrix {
        let mut result = Matrix::identity();
        let cos = theta.cos();
        let sin = theta.sin();
        result.set_index(0, 0, cos);
        result.set_index(0, 1, -sin);
        result.set_index(1, 0, sin);
        result.set_index(1, 1, cos);
        return result;
    }

    pub fn shearing_matrix(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Matrix {
        let mut result = Matrix::identity();
        result.set_index(0, 1, xy);
        result.set_index(0, 2, xz);
        result.set_index(1, 0, yx);
        result.set_index(1, 2, yz);
        result.set_index(2, 0, zx);
        result.set_index(2, 1, zy);
        return result;
    }

    pub fn view_transform(from: Tuple, to: Tuple, up: Tuple) -> Matrix {
        let forward = (to - from).normalize();
        let up_normalized = up.normalize();
        let left_vector = forward.cross(&up_normalized);
        let true_up = left_vector.cross(&forward);
        let orientation = Transformations::new(vec![
            left_vector.x, left_vector.y, left_vector.z, 0.0,
            true_up.x, true_up.y, true_up.z, 0.0,
            -forward.x, -forward.y, -forward.z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        return orientation * Transformations::translation(-from.x, -from.y, -from.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_translation() {
        let translation1 = Transformations::translation(5.0, -3.0, 2.0);
        let point1 = Tuple::point(-3.0, 4.0, 5.0);
        assert_eq!(translation1 * point1, Tuple::point(2.0, 1.0, 7.0));
    }

    #[test]
    fn point_translation_by_inverse() {
        let translation1 = Transformations::translation(5.0, -3.0, 2.0).inverse();
        let point1 = Tuple::point(-3.0, 4.0, 5.0);
        assert_eq!(translation1 * point1, Tuple::point(-8.0, 7.0, 3.0));
    }

    #[test]
    fn vector_translation() {
        let translation1 = Transformations::translation(5.0, -3.0, 2.0);
        let vector1 = Tuple::vector(-3.0, 4.0, 5.0);
        assert_eq!(translation1 * vector1, vector1);
    }

    #[test]
    fn point_scaling() {
        let scaling1 = Transformations::scaling(2.0, 3.0, 4.0);
        let point1 = Tuple::point(-4.0, 6.0, 8.0);
        assert_eq!(scaling1 * point1, Tuple::point(-8.0, 18.0, 32.0));
    }

    #[test]
    fn vector_scaling() {
        let scaling1 = Transformations::scaling(2.0, 3.0, 4.0);
        let vector1 = Tuple::vector(-4.0, 6.0, 8.0);
        assert_eq!(scaling1 * vector1, Tuple::vector(-8.0, 18.0, 32.0));
    }

    #[test]
    fn vector_scaling_by_inverse() {
        let scaling1 = Transformations::scaling(2.0, 3.0, 4.0).inverse();
        let vector1 = Tuple::vector(-4.0, 6.0, 8.0);
        assert_eq!(scaling1 * vector1, Tuple::vector(-2.0, 2.0, 2.0));
    }

    #[test]
    fn point_reflection() {
        let reflection1 = Transformations::scaling(-1.0, 1.0, 1.0);
        let point1 = Tuple::point(2.0, 3.0, 4.0);
        assert_eq!(reflection1 * point1, Tuple::point(-2.0, 3.0, 4.0));
    }

    #[test]
    fn point_rotation_around_x() {
        let point1 = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter_rotation1 = Transformations::rotation_x(crate::PI / 4.0);
        let full_quarter_rotation1 = Transformations::rotation_x(crate::PI / 2.0);
        assert_eq!(half_quarter_rotation1 * point1, Tuple::point(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
        assert_eq!(full_quarter_rotation1 * point1, Tuple::point(0.0, 0.0, 1.0));
    }

    #[test]
    fn point_rotation_inverse_around_x() {
        let point1 = Tuple::point(0.0, 1.0, 0.0);
        let full_quarter_rotation_inverse1 = Transformations::rotation_x(crate::PI / 4.0).inverse();
        assert_eq!(full_quarter_rotation_inverse1 * point1, Tuple::point(0.0, 2.0_f64.sqrt() / 2.0, -(2.0_f64.sqrt()) / 2.0));
    }

    #[test]
    fn point_rotation_around_y() {
        let point1 = Tuple::point(0.0, 0.0, 1.0);
        let half_quarter_rotation1 = Transformations::rotation_y(crate::PI / 4.0);
        let full_quarter_rotation1 = Transformations::rotation_y(crate::PI / 2.0);
        assert_eq!(half_quarter_rotation1 * point1, Tuple::point(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0));
        assert_eq!(full_quarter_rotation1 * point1, Tuple::point(1.0, 0.0, 0.0));
    }

    #[test]
    fn point_rotation_around_z() {
        let point1 = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter_rotation1 = Transformations::rotation_z(crate::PI / 4.0);
        let full_quarter_rotation1 = Transformations::rotation_z(crate::PI / 2.0);
        assert_eq!(half_quarter_rotation1 * point1, Tuple::point(-(2.0_f64.sqrt()) / 2.0, 2.0_f64.sqrt() / 2.0, 0.0));
        assert_eq!(full_quarter_rotation1 * point1, Tuple::point(-1.0, 0.0, 0.0));
    }

    #[test]
    fn point_shearing() {
        let shearing1 = Transformations::shearing_matrix(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let point1 = Tuple::point(2.0, 3.0, 4.0);
        assert_eq!(shearing1 * point1, Tuple::point(5.0, 3.0, 4.0));

        let shearing1 = Transformations::shearing_matrix(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let point1 = Tuple::point(2.0, 3.0, 4.0);
        assert_eq!(shearing1 * point1, Tuple::point(6.0, 3.0, 4.0));

        let shearing1 = Transformations::shearing_matrix(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let point1 = Tuple::point(2.0, 3.0, 4.0);
        assert_eq!(shearing1 * point1, Tuple::point(2.0, 5.0, 4.0));

        let shearing1 = Transformations::shearing_matrix(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let point1 = Tuple::point(2.0, 3.0, 4.0);
        assert_eq!(shearing1 * point1, Tuple::point(2.0, 7.0, 4.0));

        let shearing1 = Transformations::shearing_matrix(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let point1 = Tuple::point(2.0, 3.0, 4.0);
        assert_eq!(shearing1 * point1, Tuple::point(2.0, 3.0, 6.0));

        let shearing1 = Transformations::shearing_matrix(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let point1 = Tuple::point(2.0, 3.0, 4.0);
        assert_eq!(shearing1 * point1, Tuple::point(2.0, 3.0, 7.0));
    }

    #[test]
    fn transformation_order() {
        let point1 = Tuple::point(1.0, 0.0, 1.0);
        let rotation1 = Transformations::rotation_x(crate::PI / 2.0);
        let scaling1 = Transformations::scaling(5.0, 5.0, 5.0);
        let translation1 = Transformations::translation(10.0, 5.0, 7.0);
        let point2 = rotation1.clone() * point1;
        assert_eq!(point2, Tuple::point(1.0, -1.0, 0.0));
        let point3 = scaling1.clone() * point2;
        assert_eq!(point3, Tuple::point(5.0, -5.0, 0.0));
        let point4 = translation1.clone() * point3;
        assert_eq!(point4, Tuple::point(15.0, 0.0, 7.0));

        let total_transform = translation1 * scaling1 * rotation1;
        let point5 = total_transform * point1;
        assert_eq!(point5, Tuple::point(15.0, 0.0, 7.0));
    }

    #[test]
    fn transformation_matrix_for_default_orientation() {
        let from = Tuple::point(0.0, 0.0, 0.0);
        let to = Tuple::point(0.0, 0.0, -1.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let orientation = Transformations::view_transform(from, to, up);
        assert_eq!(orientation, Transformations::identity());
    }

    #[test]
    fn transformation_matrix_with_positive_z_direction() {
        let from = Tuple::point(0.0, 0.0, 0.0);
        let to = Tuple::point(0.0, 0.0, 1.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let orientation = Transformations::view_transform(from, to, up);
        assert_eq!(orientation, Transformations::scaling(-1.0, 1.0, -1.0));
    }

    #[test]
    fn view_transformation_moves_world() {
        let from = Tuple::point(0.0, 0.0, 8.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let orientation = Transformations::view_transform(from, to, up);
        assert_eq!(orientation, Transformations::translation(0.0, 0.0, -8.0));
    }

    #[test]
    fn complex_view_tranformation() {
        let from = Tuple::point(1.0, 3.0, 2.0);
        let to = Tuple::point(4.0, -2.0, 8.0);
        let up = Tuple::vector(1.0, 1.0, 0.0);
        let orientation = Transformations::view_transform(from, to, up);
        let result = Transformations::new(vec![
            -0.5070925528371099, 0.5070925528371099, 0.6761234037828132, -2.366431913239846,
            0.7677159338596801, 0.6060915267313263, 0.12121830534626524, -2.8284271247461894,
            -0.35856858280031806, 0.5976143046671968, -0.7171371656006361, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        assert_eq!(orientation, result);
    }
}
