use std::ops::Mul;
use crate::{Tuple, TupleTrait};

trait MatrixTrait: Clone + Copy + Mul + PartialEq {
    fn new(content: Vec<f64>) -> Self;
    fn get_index(&self, row: usize, column: usize) -> f64;
    fn set_index(&mut self, row: usize, column: usize, value: f64);
    fn identity() -> Self;
    fn transpose(&self) -> Self;
    fn determinant(&self) -> f64;
    fn submatrix(&self, row: usize, column: usize) -> Self;
    fn minor(&self, row: usize, column: usize) -> f64;
    fn cofactor(&self, row: usize, column: usize) -> f64;
}

#[derive(Clone, Debug)]
pub struct Matrix {
    elements: Vec<f64>,
    side_length: usize,
}

impl Matrix {
    pub fn new<T: Into<Vec<f64>>>(content: T) -> Matrix {
        let local_content = content.into();
        let side_length = (local_content.len() as f64).sqrt();
        if side_length.trunc() != side_length {
            panic!("Matrix must be square");
        }
        return Matrix { elements: local_content, side_length: side_length as usize };
    }

    pub fn empty(size: usize) -> Matrix {
        return Matrix::new(vec![0.0; size]);
    }

    pub fn get_index(&self, row: usize, column: usize) -> f64 {
        return self.elements[column + row * self.side_length];
    }

    pub fn set_index(&mut self, row: usize, column: usize, value: f64) {
        self.elements[column + row * self.side_length] = value;
    }

    pub fn identity() -> Matrix {
        return Matrix::new(vec![
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
    }

    pub fn transpose(&self) -> Matrix {
        let mut result = self.clone();
        let side_length = result.side_length;
        for row in 0..side_length {
            for column in row..side_length {
                result.elements.swap(column + row * side_length, row + column * side_length);
            }
        }
        return result;
    }

    fn determinant(&self) -> f64 {
        if self.side_length == 2 {
            return self.elements[0] * self.elements[3] - self.elements[1] * self.elements[2];
        }

        let mut determinant = 0.0;
        for column in 0..self.side_length {
            determinant += self.get_index(0, column) * self.cofactor(0, column);
        }
        return determinant;
    }

    fn submatrix(&self, row: usize, column: usize) -> Matrix {
        let mut submatrix = Vec::new();

        for row_index in 0..self.side_length {
            if row_index != row {
                for column_index in 0..self.side_length {
                    if column_index != column {
                        submatrix.push(self.get_index(row_index, column_index));
                    }
                }
            }
        }
        return Matrix::new(submatrix);
    }

    pub fn minor(&self, row: usize, column: usize) -> f64 {
        return self.submatrix(row, column).determinant();
    }

    pub fn cofactor(&self, row: usize, column: usize) -> f64 {
        let minor = self.minor(row, column);
        return if (row + column) % 2 == 0 { minor } else { -minor };
    }

    pub fn revertible(&self) -> bool {
        return self.determinant() != 0.0;
    }

    pub fn inverse(&self) -> Matrix {
        let determinant = self.determinant();
        if determinant == 0.0 {
            panic!("Matrix is not revertible");
        }

        let mut result = Matrix::empty(self.elements.len());

        for row in 0..self.side_length {
            for column in 0..self.side_length {
                let cofactor = self.cofactor(row, column);
                result.set_index(column, row, cofactor / determinant);
            }
        }

        return result;
    }
}

impl Default for Matrix {
    fn default() -> Matrix {
        return Matrix::identity();
    }
}

impl PartialEq for Matrix {
    fn eq(&self, rhs: &Matrix) -> bool {
        for (a, b) in self.elements.iter().zip(rhs.elements.iter()) {
            if (a - b).abs() > crate::EPSILON {
                return false;
            }
        }
        return true;
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Matrix {
        let mut result = Matrix::new(vec![0.0; self.elements.len()]);
        let side_length = self.side_length;
        for (index, value) in result.elements.iter_mut().enumerate() {
            let row = index / side_length;
            let column = index % side_length;
            let mut sum = 0.0;
            for i in 0..side_length {
                sum += self.get_index(row, i) * rhs.get_index(i, column);
            }
            *value = sum;
        }

        return result;
    }
}

impl Mul<Tuple> for Matrix {
    type Output = Tuple;

    fn mul(self, rhs: Tuple) -> Tuple {
        if self.side_length != 4 {
            panic!("Matrix must be 4x4 to be multiplied by a tuple");
        }

        let mut result = [0.0; 4];
        let tuple_values = rhs.get_values();
        for i in 0..4 {
            let mut sum = 0.0;
            for j in 0..4 {
                sum += self.get_index(i, j) * tuple_values[j];
            }
            result[i] = sum;
        }
        return Tuple::new(result[0], result[1], result[2], result[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_indices() {
        let matrix = Matrix::new(vec![
            1.0, 2.0, 3.0, 4.0,
            5.5, 6.5, 7.5, 8.5,
            9.0, 10.0, 11.0, 12.0,
            13.5, 14.5, 15.5, 16.5,
        ]);
        assert_eq!(matrix.get_index(0, 0), 1.0);
        assert_eq!(matrix.get_index(0, 3), 4.0);
        assert_eq!(matrix.get_index(1, 0), 5.5);
        assert_eq!(matrix.get_index(1, 2), 7.5);
        assert_eq!(matrix.get_index(2, 2), 11.0);
        assert_eq!(matrix.get_index(3, 0), 13.5);
        assert_eq!(matrix.get_index(3, 2), 15.5);
    }

    #[test]
    fn matrix_comparison() {
        let matrix1 = Matrix::new(vec![
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 8.0, 7.0, 6.0,
            5.0, 4.0, 3.0, 2.0,
        ]);
        let matrix2 = matrix1.clone();
        let matrix3 = Matrix::new(vec![
            2.0, 3.0, 4.0, 5.0,
            6.0, 7.0, 8.0, 9.0,
            8.0, 7.0, 6.0, 5.0,
            4.0, 3.0, 2.0, 1.0,
        ]);
        assert_eq!(matrix1, matrix2);
        assert_ne!(matrix2, matrix3);
    }

    #[test]
    fn matrix_multiplication_by_matrix() {
        let matrix1 = Matrix::new(vec![
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 8.0, 7.0, 6.0,
            5.0, 4.0, 3.0, 2.0,
        ]);
        let matrix2 = Matrix::new(vec![
            -2.0, 1.0, 2.0, 3.0,
            3.0, 2.0, 1.0, -1.0,
            4.0, 3.0, 6.0, 5.0,
            1.0, 2.0, 7.0, 8.0,
        ]);
        let result_matrix = Matrix::new(vec![
            20.0, 22.0, 50.0, 48.0,
            44.0, 54.0, 114.0, 108.0,
            40.0, 58.0, 110.0, 102.0,
            16.0, 26.0, 46.0, 42.0,
        ]);
        assert_eq!(matrix1 * matrix2, result_matrix);
    }

    #[test]
    fn matrix_multiplication_by_tuple() {
        let matrix = Matrix::new(vec![
            1.0, 2.0, 3.0, 4.0,
            2.0, 4.0, 4.0, 2.0,
            8.0, 6.0, 4.0, 1.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
        let tuple = Tuple::new(1.0, 2.0, 3.0, 1.0);
        assert_eq!(matrix * tuple, Tuple::new(18.0, 24.0, 33.0, 1.0));
    }

    #[test]
    fn identity_matrix() {
        let matrix1 = Matrix::new(vec![
            0.0, 1.0, 2.0, 4.0,
            1.0, 2.0, 4.0, 8.0,
            2.0, 4.0, 8.0, 16.0,
            4.0, 8.0, 16.0, 32.0,
        ]);
        assert_eq!((matrix1.clone() * Matrix::identity()), matrix1);
    }

    #[test]
    fn transpose_matrix() {
        let matrix1 = Matrix::new(vec![
            0.0, 9.0, 3.0, 0.0,
            9.0, 8.0, 0.0, 8.0,
            1.0, 8.0, 5.0, 3.0,
            0.0, 0.0, 5.0, 8.0,
        ]);
        let matrix2 = Matrix::new(vec![
            0.0, 9.0, 1.0, 0.0,
            9.0, 8.0, 8.0, 0.0,
            3.0, 0.0, 5.0, 5.0,
            0.0, 8.0, 3.0, 8.0,
        ]);
        assert_eq!(matrix1.transpose(), matrix2);
    }

    #[test]
    fn transpose_identity_matrix() {
        let matrix1 = Matrix::identity();
        assert_eq!(matrix1.transpose(), Matrix::identity());
    }

    #[test]
    fn matrix_determinant_2x2() {
        let matrix1 = Matrix::new(vec![
            1.0, 5.0,
            -3.0, 2.0,
        ]);
        assert_eq!(matrix1.determinant(), 17.0);
    }

    #[test]
    fn matrix_determinant_3x3() {
        let matrix1 = Matrix::new(vec![
            1.0, 2.0, 6.0,
            -5.0, 8.0, -4.0,
            2.0, 6.0, 4.0,
        ]);
        assert_eq!(matrix1.cofactor(0, 0), 56.0);
        assert_eq!(matrix1.cofactor(0, 1), 12.0);
        assert_eq!(matrix1.cofactor(0, 2), -46.0);
        assert_eq!(matrix1.determinant(), -196.0);
    }

    #[test]
    fn matrix_determinant_4x4() {
        let matrix1 = Matrix::new(vec![
            -2.0, -8.0, 3.0, 5.0,
            -3.0, 1.0, 7.0, 3.0,
            1.0, 2.0, -9.0, 6.0,
            -6.0, 7.0, 7.0, -9.0,
        ]);
        assert_eq!(matrix1.cofactor(0, 0), 690.0);
        assert_eq!(matrix1.cofactor(0, 1), 447.0);
        assert_eq!(matrix1.cofactor(0, 2), 210.0);
        assert_eq!(matrix1.cofactor(0, 3), 51.0);
        assert_eq!(matrix1.determinant(), -4071.0);
    }

    #[test]
    fn submatrix_3x3() {
        let matrix1 = Matrix::new(vec![
            1.0, 5.0, 0.0,
            -3.0, 2.0, 7.0,
            0.0, 6.0, -3.0,
        ]);
        let matrix2 = Matrix::new(vec![
            -3.0, 2.0,
            0.0, 6.0,
        ]);
        assert_eq!(matrix1.submatrix(0, 2), matrix2);
    }

    #[test]
    fn submatrix_4x4() {
        let matrix1 = Matrix::new(vec![
            -6.0, 1.0, 1.0, 6.0,
            -8.0, 5.0, 8.0, 6.0,
            -1.0, 0.0, 8.0, 2.0,
            -7.0, 1.0, -1.0, 1.0,
        ]);
        let matrix2 = Matrix::new(vec![
            -6.0, 1.0, 6.0,
            -8.0, 8.0, 6.0,
            -7.0, -1.0, 1.0,
        ]);
        assert_eq!(matrix1.submatrix(2, 1), matrix2);
    }

    #[test]
    fn matrix_minor_and_cofactor() {
        let matrix1 = Matrix::new(vec![
            3.0, 5.0, 0.0,
            2.0, -1.0, -7.0,
            6.0, -1.0, 5.0,
        ]);
        assert_eq!(matrix1.minor(0, 0), -12.0);
        assert_eq!(matrix1.cofactor(0, 0), -12.0);
        assert_eq!(matrix1.minor(1, 0), 25.0);
        assert_eq!(matrix1.cofactor(1, 0), -25.0);
    }

    #[test]
    fn matrix_inverse() {
        let matrix1 = Matrix::new(vec![
            -5.0, 2.0, 6.0, -8.0,
            1.0, -5.0, 1.0, 8.0,
            7.0, 7.0, -6.0, -7.0,
            1.0, -3.0, 7.0, 4.0,
        ]);
        let matrix2 = Matrix::new(vec![
            0.21804511278195488, 0.45112781954887216, 0.24060150375939848, -0.045112781954887216,
            -0.8082706766917294, -1.4567669172932332, -0.44360902255639095, 0.5206766917293233,
            -0.07894736842105263, -0.2236842105263158, -0.05263157894736842, 0.19736842105263158,
            -0.5225563909774437, -0.8139097744360902, -0.3007518796992481, 0.30639097744360905,
        ]);
        let matrix3 = Matrix::new(vec![
            9.0, 3.0, 0.0, 9.0,
            -5.0, -2.0, 6.0, -3.0,
            -4.0, 9.0, 6.0, 4.0,
            -7.0, 6.0, 6.0, 2.0,
        ]);
        let matrix4 = Matrix::new(vec![
            -0.004901960784313725, 0.10294117647058823, 0.27941176470588236, -0.38235294117647056,
            -0.09313725490196079, -0.04411764705882353, 0.3088235294117647, -0.2647058823529412,
            0.03839869281045752, 0.19362745098039216, 0.14460784313725492, -0.1715686274509804,
            0.14705882352941177, -0.08823529411764706, -0.38235294117647056, 0.47058823529411764,
        ]);
        let matrix5 = matrix1.inverse();
        assert_eq!(matrix1.determinant(), 532.0);
        assert_eq!(matrix1.cofactor(2, 3), -160.0);
        assert_eq!(matrix5.get_index(3, 2), -160.0 / 532.0);
        assert_eq!(matrix1.cofactor(3, 2), 105.0);
        assert_eq!(matrix5.get_index(2, 3), 105.0 / 532.0);
        assert_eq!(matrix5, matrix2);
        assert_eq!(matrix3.inverse(), matrix4);
    }

    #[test]
    fn matrix_product_mul_by_inverse() {
        let matrix1 = Matrix::new(vec![
            3.0, -9.0, 7.0, 3.0,
            3.0, -8.0, 2.0, -9.0,
            -4.0, 4.0, 4.0, 1.0,
            -6.0, 5.0, -1.0, 1.0,
        ]);
        let matrix2 = Matrix::new(vec![
            8.0, 2.0, 2.0, 2.0,
            3.0, -1.0, 7.0, 0.0,
            7.0, 0.0, 5.0, 4.0,
            6.0, -2.0, 0.0, 5.0,
        ]);
        let matrix3 = matrix1.clone() * matrix2.clone();
        assert_eq!(matrix3 * matrix2.inverse(), matrix1);
    }
}
