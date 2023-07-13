use crate::consts::EPSILON;
use crate::primitives::{Point, Vector};
use bincode::Encode;
use std::fmt::{Display, Formatter};
use std::mem;
use std::ops::Mul;

#[derive(Clone, Copy, Debug, Encode)]
pub struct Matrix<const SIDE_LENGTH: usize> {
    elements: [[f64; SIDE_LENGTH]; SIDE_LENGTH],
}

impl<const SIDE_LENGTH: usize> Matrix<SIDE_LENGTH> {
    pub const EMPTY: Matrix<SIDE_LENGTH> = Matrix {
        elements: [[0.0; SIDE_LENGTH]; SIDE_LENGTH],
    };

    pub const IDENTITY: Matrix<SIDE_LENGTH> = {
        let mut result = Self::EMPTY;
        let mut index = 0;
        while index < SIDE_LENGTH {
            let mut tmp_row = [0.0; SIDE_LENGTH];
            tmp_row[index] = 1.0;
            result.elements[index] = tmp_row;
            index += 1;
        }
        result
    };

    pub fn new(elements: [[f64; SIDE_LENGTH]; SIDE_LENGTH]) -> Matrix<SIDE_LENGTH> {
        return Matrix { elements };
    }

    pub fn get_index(&self, row: usize, column: usize) -> f64 {
        return self.elements[row][column];
    }

    pub fn set_index(&mut self, row: usize, column: usize, value: f64) {
        self.elements[row][column] = value;
    }

    #[inline(always)]
    pub fn transpose(&self) -> Matrix<SIDE_LENGTH> {
        let mut result = *self;
        for row in 0..SIDE_LENGTH {
            let row_offset = row + 1;
            let (upper_slice, lower_slice) = result.elements.split_at_mut(row_offset);
            for column in row_offset..SIDE_LENGTH {
                mem::swap(
                    &mut upper_slice[row][column],
                    &mut lower_slice[column - row_offset][row],
                );
            }
        }
        return result;
    }
}

impl Matrix<2> {
    #[inline(always)]
    pub fn submatrix(&self, row: usize, column: usize) -> f64 {
        return self.elements[row][column];
    }

    #[inline(always)]
    fn determinant(&self) -> f64 {
        return self.elements[0][0] * self.elements[1][1]
            - self.elements[0][1] * self.elements[1][0];
    }

    pub fn minor(&self, row: usize, column: usize) -> f64 {
        return self.submatrix(row, column);
    }

    pub fn cofactor(&self, row: usize, column: usize) -> f64 {
        let minor = self.minor(row, column);
        return if (row + column) % 2 == 0 {
            minor
        } else {
            -minor
        };
    }

    pub fn revertible(&self) -> bool {
        return self.determinant() != 0.0;
    }

    #[inline(always)]
    pub fn inverse(&self) -> Matrix<2> {
        let mut result = Matrix::<2>::EMPTY;
        let determinant = self.determinant();

        for row in 0..2 {
            for column in 0..2 {
                let cofactor = self.cofactor(row, column);
                result.set_index(column, row, cofactor / determinant);
            }
        }

        return result;
    }
}

impl Matrix<3> {
    #[inline(always)]
    fn submatrix(&self, row: usize, column: usize) -> Matrix<2> {
        let mut result = Matrix::<2>::EMPTY;

        for row_index in 0..self.elements.len() {
            if row_index != row {
                for column_index in 0..self.elements.len() {
                    if column_index != column {
                        let new_row_index = if row_index < row {
                            row_index
                        } else {
                            row_index - 1
                        };
                        let new_column_index = if column_index < column {
                            column_index
                        } else {
                            column_index - 1
                        };
                        result.elements[new_row_index][new_column_index] =
                            self.elements[row_index][column_index];
                    }
                }
            }
        }
        return result;
    }

    #[inline(always)]
    fn determinant(&self) -> f64 {
        let mut determinant = 0.0;
        for column in 0..self.elements.len() {
            determinant += self.elements[0][column] * self.cofactor(0, column);
        }
        return determinant;
    }

    pub fn minor(&self, row: usize, column: usize) -> f64 {
        return self.submatrix(row, column).determinant();
    }

    pub fn cofactor(&self, row: usize, column: usize) -> f64 {
        let minor = self.minor(row, column);
        return if (row + column) % 2 == 0 {
            minor
        } else {
            -minor
        };
    }

    pub fn revertible(&self) -> bool {
        return self.determinant() != 0.0;
    }

    #[inline(always)]
    pub fn inverse(&self) -> Matrix<3> {
        let mut result = Matrix::<3>::EMPTY;
        let determinant = self.determinant();

        for row in 0..3 {
            for column in 0..3 {
                let cofactor = self.cofactor(row, column);
                result.set_index(column, row, cofactor / determinant);
            }
        }

        return result;
    }
}

impl Matrix<4> {
    #[inline(always)]
    fn submatrix(&self, row: usize, column: usize) -> Matrix<3> {
        let mut result = Matrix::<3>::EMPTY;

        for row_index in 0..self.elements.len() {
            if row_index != row {
                for column_index in 0..self.elements.len() {
                    if column_index != column {
                        let new_row_index = if row_index < row {
                            row_index
                        } else {
                            row_index - 1
                        };
                        let new_column_index = if column_index < column {
                            column_index
                        } else {
                            column_index - 1
                        };
                        result.elements[new_row_index][new_column_index] =
                            self.elements[row_index][column_index];
                    }
                }
            }
        }
        return result;
    }

    #[inline(always)]
    fn determinant(&self) -> f64 {
        let mut determinant = 0.0;
        for column in 0..self.elements.len() {
            determinant += self.elements[0][column] * self.cofactor(0, column);
        }
        return determinant;
    }

    fn minor(&self, row: usize, column: usize) -> f64 {
        return self.submatrix(row, column).determinant();
    }

    fn cofactor(&self, row: usize, column: usize) -> f64 {
        let minor = self.minor(row, column);
        return if (row + column) % 2 == 0 {
            minor
        } else {
            -minor
        };
    }

    pub fn revertible(&self) -> bool {
        return self.determinant() != 0.0;
    }

    #[inline(always)]
    pub fn inverse(&self) -> Matrix<4> {
        let mut result = Matrix::<4>::EMPTY;
        let determinant = self.determinant();
        for row in 0..4 {
            for column in 0..4 {
                result.set_index(column, row, self.cofactor(row, column) / determinant);
            }
        }
        return result;
        // the following caching approach causes performance regression
        // thread_local! {
        //     static CACHE: RefCell<HashMap<Matrix<4>, Matrix<4>>> = RefCell::new(HashMap::with_capacity(1024));
        // }
        // return CACHE.with(|cache| {
        //     let mut cache = cache.borrow_mut();
        //     let cached = cache.get(self);
        //     match cached {
        //         Some(value) => {
        //             return value.clone();
        //         }
        //         None => {
        //             let mut result = Matrix::<4>::empty();
        //             let determinant = self.determinant();
        //
        //             for row in 0..4 {
        //                 for column in 0..4 {
        //                     let cofactor = self.cofactor(row, column);
        //                     result.set_index(column, row, cofactor / determinant);
        //                 }
        //             }
        //             cache.insert(*self, result);
        //             return result;
        //         }
        //     }
        // });
    }
}

impl<const SIDE_LENGTH: usize> Default for Matrix<SIDE_LENGTH> {
    fn default() -> Matrix<SIDE_LENGTH> {
        return Matrix::IDENTITY;
    }
}

impl<const SIDE_LENGTH: usize> Display for Matrix<SIDE_LENGTH> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        return formatter
            .debug_struct("Matrix")
            .field("elements", &self.elements)
            .finish();
    }
}

impl<const SIDE_LENGTH: usize> PartialEq for Matrix<SIDE_LENGTH> {
    fn eq(&self, rhs: &Matrix<SIDE_LENGTH>) -> bool {
        for (self_row, rhs_row) in self.elements.iter().zip(rhs.elements.iter()) {
            for (self_value, rhs_value) in self_row.iter().zip(rhs_row.iter()) {
                if (self_value - rhs_value).abs() > EPSILON {
                    return false;
                }
            }
        }
        return true;
    }
}

// impl<const SIDE_LENGTH: usize> Hash for Matrix<SIDE_LENGTH> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         for row in self.elements {
//             for value in row {
//                 unsafe { mem::transmute::<f64, u64>(value).hash(state) };
//             }
//         }
//     }
// }

impl<const SIDE_LENGTH: usize> Mul for Matrix<SIDE_LENGTH> {
    type Output = Matrix<SIDE_LENGTH>;

    fn mul(self, rhs: Matrix<SIDE_LENGTH>) -> Self::Output {
        let mut result = Matrix::<SIDE_LENGTH>::EMPTY;
        for (row_index, row) in result.elements.iter_mut().enumerate() {
            for (column_index, value) in row.iter_mut().enumerate() {
                let mut sum = 0.0;
                for i in 0..SIDE_LENGTH {
                    sum += self.get_index(row_index, i) * rhs.get_index(i, column_index);
                }
                *value = sum;
            }
        }

        return result;
    }
}

impl Mul<Point> for Matrix<4> {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        let mut result = [0.0; 4];
        let point_values = rhs.get_values();
        for (index, row) in self.elements.iter().enumerate() {
            let mut sum = 0.0;
            for (matrix_value, point_value) in row.iter().zip(point_values) {
                sum += matrix_value * point_value;
            }
            result[index] = sum;
        }
        return Point::new(result[0], result[1], result[2]);
    }
}

impl Mul<Vector> for Matrix<4> {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        let mut result = [0.0; 4];
        let vector_values = rhs.get_values();
        for (index, row) in self.elements.iter().enumerate() {
            let mut sum = 0.0;
            for (matrix_value, vector_value) in row.iter().zip(vector_values) {
                sum += matrix_value * vector_value;
            }
            result[index] = sum;
        }

        return Vector::new(result[0], result[1], result[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_indices() {
        let matrix_2 = Matrix::<2>::new([[1.0, 2.0], [5.5, 6.5]]);
        assert_eq!(matrix_2.get_index(0, 0), 1.0);
        assert_eq!(matrix_2.get_index(0, 1), 2.0);
        assert_eq!(matrix_2.get_index(1, 0), 5.5);
        assert_eq!(matrix_2.get_index(1, 1), 6.5);

        let matrix_3 = Matrix::<3>::new([[1.0, 2.0, 3.0], [5.5, 6.5, 7.5], [9.0, 10.0, 11.0]]);
        assert_eq!(matrix_3.get_index(0, 0), 1.0);
        assert_eq!(matrix_3.get_index(0, 1), 2.0);
        assert_eq!(matrix_3.get_index(1, 0), 5.5);
        assert_eq!(matrix_3.get_index(1, 1), 6.5);
        assert_eq!(matrix_3.get_index(2, 2), 11.0);

        let matrix_4 = Matrix::<4>::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.5, 6.5, 7.5, 8.5],
            [9.0, 10.0, 11.0, 12.0],
            [13.5, 14.5, 15.5, 16.5],
        ]);
        assert_eq!(matrix_4.get_index(0, 0), 1.0);
        assert_eq!(matrix_4.get_index(0, 3), 4.0);
        assert_eq!(matrix_4.get_index(1, 0), 5.5);
        assert_eq!(matrix_4.get_index(1, 2), 7.5);
        assert_eq!(matrix_4.get_index(2, 2), 11.0);
        assert_eq!(matrix_4.get_index(3, 0), 13.5);
        assert_eq!(matrix_4.get_index(3, 2), 15.5);
    }

    #[test]
    fn matrix_comparison() {
        let matrix_1 = Matrix::<4>::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let matrix_2 = matrix_1;
        let matrix_3 = Matrix::<4>::new([
            [2.0, 3.0, 4.0, 5.0],
            [6.0, 7.0, 8.0, 9.0],
            [8.0, 7.0, 6.0, 5.0],
            [4.0, 3.0, 2.0, 1.0],
        ]);
        assert_eq!(matrix_1, matrix_2);
        assert_ne!(matrix_2, matrix_3);
    }

    #[test]
    fn matrix_multiplication_by_matrix() {
        let matrix_1 = Matrix::<4>::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let matrix_2 = Matrix::<4>::new([
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ]);
        let result_matrix = Matrix::<4>::new([
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ]);
        assert_eq!(matrix_1 * matrix_2, result_matrix);
    }

    #[test]
    fn matrix_multiplication_by_point() {
        let matrix = Matrix::<4>::new([
            [1.0, 2.0, 3.0, 4.0],
            [2.0, 4.0, 4.0, 2.0],
            [8.0, 6.0, 4.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let point = Point::new(1.0, 2.0, 3.0);
        assert_eq!(matrix * point, Point::new(18.0, 24.0, 33.0));
    }

    #[test]
    fn identity_matrix() {
        let matrix_2 = Matrix::<2>::new([[0.0, 1.0], [1.0, 2.0]]);
        let matrix_3 = Matrix::<3>::new([[0.0, 1.0, 2.0], [1.0, 2.0, 4.0], [2.0, 4.0, 8.0]]);
        let matrix_4 = Matrix::<4>::new([
            [0.0, 1.0, 2.0, 4.0],
            [1.0, 2.0, 4.0, 8.0],
            [2.0, 4.0, 8.0, 16.0],
            [4.0, 8.0, 16.0, 32.0],
        ]);
        assert_eq!((matrix_2 * Matrix::<2>::IDENTITY), matrix_2);
        assert_eq!((matrix_3 * Matrix::<3>::IDENTITY), matrix_3);
        assert_eq!((matrix_4 * Matrix::<4>::IDENTITY), matrix_4);
    }

    #[test]
    fn transpose_matrix() {
        let matrix_2 = Matrix::<2>::new([[1.0, 2.0], [5.5, 6.5]]);
        let matrix_2_transposed = Matrix::<2>::new([[1.0, 5.5], [2.0, 6.5]]);
        let matrix_3 = Matrix::<3>::new([[0.0, 9.0, 3.0], [9.0, 8.0, 0.0], [1.0, 8.0, 5.0]]);
        let matrix_3_transposed =
            Matrix::<3>::new([[0.0, 9.0, 1.0], [9.0, 8.0, 8.0], [3.0, 0.0, 5.0]]);
        let matrix_4 = Matrix::<4>::new([
            [0.0, 9.0, 3.0, 0.0],
            [9.0, 8.0, 0.0, 8.0],
            [1.0, 8.0, 5.0, 3.0],
            [0.0, 0.0, 5.0, 8.0],
        ]);
        let matrix_4_transposed = Matrix::<4>::new([
            [0.0, 9.0, 1.0, 0.0],
            [9.0, 8.0, 8.0, 0.0],
            [3.0, 0.0, 5.0, 5.0],
            [0.0, 8.0, 3.0, 8.0],
        ]);
        assert_eq!(matrix_2.transpose(), matrix_2_transposed);
        assert_eq!(matrix_3.transpose(), matrix_3_transposed);
        assert_eq!(matrix_4.transpose(), matrix_4_transposed);
    }

    #[test]
    fn transpose_identity_matrix() {
        assert_eq!(Matrix::<2>::IDENTITY.transpose(), Matrix::<2>::IDENTITY);
        assert_eq!(Matrix::<3>::IDENTITY.transpose(), Matrix::<3>::IDENTITY);
        assert_eq!(Matrix::<4>::IDENTITY.transpose(), Matrix::<4>::IDENTITY);
    }

    #[test]
    fn matrix_determinant_2x2() {
        let matrix = Matrix::<2>::new([[1.0, 5.0], [-3.0, 2.0]]);
        assert_eq!(matrix.determinant(), 17.0);
    }

    #[test]
    fn matrix_determinant_3x3() {
        let matrix = Matrix::<3>::new([[1.0, 2.0, 6.0], [-5.0, 8.0, -4.0], [2.0, 6.0, 4.0]]);
        assert_eq!(matrix.cofactor(0, 0), 56.0);
        assert_eq!(matrix.cofactor(0, 1), 12.0);
        assert_eq!(matrix.cofactor(0, 2), -46.0);
        assert_eq!(matrix.determinant(), -196.0);
    }

    #[test]
    fn matrix_determinant_4x4() {
        let matrix = Matrix::<4>::new([
            [-2.0, -8.0, 3.0, 5.0],
            [-3.0, 1.0, 7.0, 3.0],
            [1.0, 2.0, -9.0, 6.0],
            [-6.0, 7.0, 7.0, -9.0],
        ]);
        assert_eq!(matrix.cofactor(0, 0), 690.0);
        assert_eq!(matrix.cofactor(0, 1), 447.0);
        assert_eq!(matrix.cofactor(0, 2), 210.0);
        assert_eq!(matrix.cofactor(0, 3), 51.0);
        assert_eq!(matrix.determinant(), -4071.0);
    }

    #[test]
    fn submatrix_3x3() {
        let matrix_1 = Matrix::<3>::new([[1.0, 5.0, 0.], [-3.0, 2.0, 7.0], [0.0, 6.0, -3.0]]);
        let matrix_2 = Matrix::<2>::new([[-3.0, 2.0], [0.0, 6.0]]);
        assert_eq!(matrix_1.submatrix(0, 2), matrix_2);
    }

    #[test]
    fn submatrix_4x4() {
        let matrix_1 = Matrix::<4>::new([
            [-6.0, 1.0, 1.0, 6.0],
            [-8.0, 5.0, 8.0, 6.0],
            [-1.0, 0.0, 8.0, 2.],
            [-7.0, 1.0, -1.0, 1.0],
        ]);
        let matrix_2 = Matrix::<3>::new([[-6.0, 1.0, 6.0], [-8.0, 8.0, 6.0], [-7.0, -1.0, 1.0]]);
        assert_eq!(matrix_1.submatrix(2, 1), matrix_2);
    }

    #[test]
    fn matrix_minor_and_cofactor() {
        let matrix = Matrix::<3>::new([[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);
        assert_eq!(matrix.minor(0, 0), -12.0);
        assert_eq!(matrix.cofactor(0, 0), -12.0);
        assert_eq!(matrix.minor(1, 0), 25.0);
        assert_eq!(matrix.cofactor(1, 0), -25.0);
    }

    #[test]
    fn matrix_inverse() {
        let matrix_1 = Matrix::<4>::new([
            [-5.0, 2.0, 6.0, -8.0],
            [1.0, -5.0, 1.0, 8.0],
            [7.0, 7.0, -6.0, -7.0],
            [1.0, -3.0, 7.0, 4.0],
        ]);
        let matrix_2 = Matrix::<4>::new([
            [
                0.21804511278195488,
                0.45112781954887216,
                0.24060150375939848,
                -0.045112781954887216,
            ],
            [
                -0.8082706766917294,
                -1.4567669172932332,
                -0.44360902255639095,
                0.5206766917293233,
            ],
            [
                -0.07894736842105263,
                -0.2236842105263158,
                -0.05263157894736842,
                0.19736842105263158,
            ],
            [
                -0.5225563909774437,
                -0.8139097744360902,
                -0.3007518796992481,
                0.30639097744360905,
            ],
        ]);
        let matrix_3 = Matrix::<4>::new([
            [9.0, 3.0, 0.0, 9.0],
            [-5.0, -2.0, 6.0, -3.0],
            [-4.0, 9.0, 6.0, 4.0],
            [-7.0, 6.0, 6.0, 2.0],
        ]);
        let matrix_4 = Matrix::<4>::new([
            [
                -0.004901960784313725,
                0.10294117647058823,
                0.27941176470588236,
                -0.38235294117647056,
            ],
            [
                -0.09313725490196079,
                -0.04411764705882353,
                0.3088235294117647,
                -0.2647058823529412,
            ],
            [
                0.03839869281045752,
                0.19362745098039216,
                0.14460784313725492,
                -0.1715686274509804,
            ],
            [
                0.14705882352941177,
                -0.08823529411764706,
                -0.38235294117647056,
                0.47058823529411764,
            ],
        ]);
        let matrix_5 = matrix_1.inverse();
        assert_eq!(matrix_1.determinant(), 532.0);
        assert_eq!(matrix_1.cofactor(2, 3), -160.0);
        assert_eq!(matrix_5.get_index(3, 2), -160.0 / 532.0);
        assert_eq!(matrix_1.cofactor(3, 2), 105.0);
        assert_eq!(matrix_5.get_index(2, 3), 105.0 / 532.0);
        assert_eq!(matrix_5, matrix_2);
        assert_eq!(matrix_3.inverse(), matrix_4);
    }

    #[test]
    fn matrix_product_mul_by_inverse() {
        let matrix_1 = Matrix::<4>::new([
            [3.0, -9.0, 7.0, 3.0],
            [3.0, -8.0, 2.0, -9.0],
            [-4.0, 4.0, 4.0, 1.0],
            [-6.0, 5.0, -1.0, 1.0],
        ]);
        let matrix_2 = Matrix::<4>::new([
            [8.0, 2.0, 2.0, 2.0],
            [3.0, -1.0, 7.0, 0.0],
            [7.0, 0.0, 5.0, 4.0],
            [6.0, -2.0, 0.0, 5.0],
        ]);
        let matrix_3 = matrix_1 * matrix_2;
        assert_eq!(matrix_3 * matrix_2.inverse(), matrix_1);
    }
}
