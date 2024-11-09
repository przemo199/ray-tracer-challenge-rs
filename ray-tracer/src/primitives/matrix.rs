use crate::primitives::{Point, Vector};
use crate::utils::CoarseEq;
use bincode::Encode;
use core::fmt::{Display, Formatter, Result};
use core::mem;
use core::ops::{Deref, DerefMut, Mul, Neg, Range};

#[derive(Clone, Copy, Debug, Encode)]
pub struct Matrix<const SIDE_LENGTH: usize>([[f64; SIDE_LENGTH]; SIDE_LENGTH]);

impl<const SIDE_LENGTH: usize> Matrix<SIDE_LENGTH> {
    pub const EMPTY: Self = Self([[0.0; SIDE_LENGTH]; SIDE_LENGTH]);

    pub const SIDE_LENGTH_RANGE: Range<usize> = 0..SIDE_LENGTH;

    pub const IDENTITY: Self = {
        let mut result = Self::EMPTY;
        let mut index = 0;
        while index < SIDE_LENGTH {
            result.0[index][index] = 1.0;
            index += 1;
        }
        result
    };

    pub const fn new(elements: [[f64; SIDE_LENGTH]; SIDE_LENGTH]) -> Self {
        return Self(elements);
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        let mut result = *self;
        for row in Self::SIDE_LENGTH_RANGE {
            let row_offset = row + 1;
            let (upper_slice, lower_slice) = result.split_at_mut(row_offset);
            for column in row_offset..SIDE_LENGTH {
                mem::swap(
                    &mut upper_slice[row][column],
                    &mut lower_slice[column - row_offset][row],
                );
            }
        }
        return result;
    }

    pub fn is_identity(&self) -> bool {
        return self.iter().enumerate().all(|(row_index, row)| {
            row.iter().enumerate().all(|(column_index, value)| {
                return value.coarse_eq(if row_index == column_index { 1.0 } else { 0.0 });
            })
        });
    }

    #[inline]
    pub fn for_each_mut(&mut self, mut op: impl Fn(usize, usize, &mut f64)) {
        for (row_index, row) in self.iter_mut().enumerate() {
            for (column_index, value) in row.iter_mut().enumerate() {
                op(row_index, column_index, value);
            }
        }
    }
}

impl Matrix<2> {
    #[inline]
    pub fn submatrix(&self, excluded_row: usize, excluded_column: usize) -> f64 {
        for row_index in Self::SIDE_LENGTH_RANGE {
            if row_index == excluded_row {
                continue;
            }
            for column_index in Self::SIDE_LENGTH_RANGE {
                if column_index == excluded_column {
                    continue;
                }
                return self.0[row_index][column_index];
            }
        }
        unreachable!();
    }

    #[inline]
    pub fn determinant(&self) -> f64 {
        return (self[0][0] * self[1][1]) - (self[0][1] * self[1][0]);
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

    #[inline]
    pub fn inverse(&self) -> Self {
        if self.is_identity() {
            return Self::IDENTITY;
        }

        let mut result = Self::EMPTY;
        let determinant = self.determinant();

        result.for_each_mut(|row_index, column_index, value| {
            *value = self.cofactor(column_index, row_index) / determinant;
        });

        return result;
    }
}

impl Matrix<3> {
    #[inline]
    fn submatrix(&self, excluded_row: usize, excluded_column: usize) -> Matrix<2> {
        let mut result = Matrix::<2>::EMPTY;

        for row_index in Self::SIDE_LENGTH_RANGE {
            if row_index == excluded_row {
                continue;
            }
            for column_index in Self::SIDE_LENGTH_RANGE {
                if column_index == excluded_column {
                    continue;
                }
                let new_row_index = if row_index < excluded_row {
                    row_index
                } else {
                    row_index - 1
                };
                let new_column_index = if column_index < excluded_column {
                    column_index
                } else {
                    column_index - 1
                };
                result[new_row_index][new_column_index] = self[row_index][column_index];
            }
        }
        return result;
    }

    #[inline]
    pub fn determinant(&self) -> f64 {
        return Self::SIDE_LENGTH_RANGE.fold(0.0, |acc, index| {
            acc + (self[0][index] * self.cofactor(0, index))
        });
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

    #[inline]
    pub fn inverse(&self) -> Self {
        if self.is_identity() {
            return Self::IDENTITY;
        }

        let mut result = Self::EMPTY;
        let determinant = self.determinant();
        result.for_each_mut(|row_index, column_index, value| {
            *value = self.cofactor(column_index, row_index) / determinant;
        });
        return result;
    }
}

impl Matrix<4> {
    #[inline]
    fn submatrix(&self, excluded_row: usize, excluded_column: usize) -> Matrix<3> {
        if self.is_identity() {
            return Matrix::<3>::IDENTITY;
        }

        let mut result = Matrix::<3>::EMPTY;

        for row_index in Self::SIDE_LENGTH_RANGE {
            if row_index == excluded_row {
                continue;
            }
            for column_index in Self::SIDE_LENGTH_RANGE {
                if column_index == excluded_column {
                    continue;
                }
                let new_row_index = if row_index < excluded_row {
                    row_index
                } else {
                    row_index - 1
                };
                let new_column_index = if column_index < excluded_column {
                    column_index
                } else {
                    column_index - 1
                };
                result[new_row_index][new_column_index] = self[row_index][column_index];
            }
        }
        return result;
    }

    #[inline]
    pub fn determinant(&self) -> f64 {
        return Self::SIDE_LENGTH_RANGE.fold(0.0, |acc, index| {
            acc + (self[0][index] * self.cofactor(0, index))
        });
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

    #[inline]
    pub fn inverse(&self) -> Self {
        if self.is_identity() {
            return Self::IDENTITY;
        }

        let mut result = Self::EMPTY;
        let determinant = self.determinant();
        result.for_each_mut(|row_index, column_index, value| {
            *value = self.cofactor(column_index, row_index) / determinant;
        });
        return result;
    }
}

impl<const SIDE_LENGTH: usize> Default for Matrix<SIDE_LENGTH> {
    fn default() -> Self {
        return Self::IDENTITY;
    }
}

impl<const SIDE_LENGTH: usize> Deref for Matrix<SIDE_LENGTH> {
    type Target = [[f64; SIDE_LENGTH]; SIDE_LENGTH];

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl<const SIDE_LENGTH: usize> DerefMut for Matrix<SIDE_LENGTH> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl<const SIDE_LENGTH: usize> Display for Matrix<SIDE_LENGTH> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        return formatter
            .debug_struct("Matrix")
            .field("elements", &self)
            .finish();
    }
}

impl<const SIDE_LENGTH: usize> PartialEq for Matrix<SIDE_LENGTH> {
    #[inline]
    fn eq(&self, rhs: &Self) -> bool {
        if std::ptr::eq(self, rhs) {
            return true;
        }

        return self
            .iter()
            .zip(rhs.iter())
            .flat_map(|(self_row, rhs_row)| self_row.iter().zip(rhs_row.iter()))
            .all(|(self_value, rhs_value)| self_value.coarse_eq(*rhs_value));
    }
}

impl<const SIDE_LENGTH: usize> Neg for Matrix<SIDE_LENGTH> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut result = self;
        result.for_each_mut(|_, _, value| {
            *value = -*value;
        });
        return result;
    }
}

impl<const SIDE_LENGTH: usize> Mul for Matrix<SIDE_LENGTH> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Self::EMPTY;
        result.for_each_mut(|row_index, column_index, value| {
            *value = rhs.iter().enumerate().fold(0.0, |acc, (index, row)| {
                acc + (self[row_index][index] * row[column_index])
            })
        });
        return result;
    }
}

impl Mul<Point> for Matrix<4> {
    type Output = Point;

    #[inline]
    fn mul(self, rhs: Point) -> Self::Output {
        return Self::Output::from_fn(|row_index| {
            return rhs
                .into_iter()
                .enumerate()
                .fold(0.0, |acc, (col_index, value)| {
                    acc + (self[row_index][col_index] * value)
                });
        });
    }
}

impl Mul<Vector> for Matrix<4> {
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output {
        return Self::Output::from_fn(|row_index| {
            return rhs
                .into_iter()
                .enumerate()
                .fold(0.0, |acc, (col_index, value)| {
                    acc + (self[row_index][col_index] * value)
                });
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::primitives::{Matrix, Point};

    #[test]
    fn matrix_indices() {
        let matrix_2 = Matrix::new([[1.0, 2.0], [5.5, 6.5]]);
        assert_eq!(matrix_2[0][0], 1.0);
        assert_eq!(matrix_2[0][1], 2.0);
        assert_eq!(matrix_2[1][0], 5.5);
        assert_eq!(matrix_2[1][1], 6.5);

        let matrix_3 = Matrix::new([[1.0, 2.0, 3.0], [5.5, 6.5, 7.5], [9.0, 10.0, 11.0]]);
        assert_eq!(matrix_3[0][0], 1.0);
        assert_eq!(matrix_3[0][1], 2.0);
        assert_eq!(matrix_3[1][0], 5.5);
        assert_eq!(matrix_3[1][1], 6.5);
        assert_eq!(matrix_3[2][2], 11.0);

        let matrix_4 = Matrix::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.5, 6.5, 7.5, 8.5],
            [9.0, 10.0, 11.0, 12.0],
            [13.5, 14.5, 15.5, 16.5],
        ]);
        assert_eq!(matrix_4[0][0], 1.0);
        assert_eq!(matrix_4[0][3], 4.0);
        assert_eq!(matrix_4[1][0], 5.5);
        assert_eq!(matrix_4[1][2], 7.5);
        assert_eq!(matrix_4[2][2], 11.0);
        assert_eq!(matrix_4[3][0], 13.5);
        assert_eq!(matrix_4[3][2], 15.5);
    }

    #[test]
    fn matrix_comparison() {
        let matrix_1 = Matrix::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let matrix_2 = matrix_1;
        let matrix_3 = Matrix::new([
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
        let matrix_1 = Matrix::new([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 8.0, 7.0, 6.0],
            [5.0, 4.0, 3.0, 2.0],
        ]);
        let matrix_2 = Matrix::new([
            [-2.0, 1.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, -1.0],
            [4.0, 3.0, 6.0, 5.0],
            [1.0, 2.0, 7.0, 8.0],
        ]);
        let result_matrix = Matrix::new([
            [20.0, 22.0, 50.0, 48.0],
            [44.0, 54.0, 114.0, 108.0],
            [40.0, 58.0, 110.0, 102.0],
            [16.0, 26.0, 46.0, 42.0],
        ]);
        assert_eq!(matrix_1 * matrix_2, result_matrix);
    }

    #[test]
    fn matrix_multiplication_by_point() {
        let matrix = Matrix::new([
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
        let matrix_2 = Matrix::new([[0.0, 1.0], [1.0, 2.0]]);
        let matrix_3 = Matrix::new([[0.0, 1.0, 2.0], [1.0, 2.0, 4.0], [2.0, 4.0, 8.0]]);
        let matrix_4 = Matrix::new([
            [0.0, 1.0, 2.0, 4.0],
            [1.0, 2.0, 4.0, 8.0],
            [2.0, 4.0, 8.0, 16.0],
            [4.0, 8.0, 16.0, 32.0],
        ]);
        assert_eq!(matrix_2 * Matrix::IDENTITY, matrix_2);
        assert_eq!(matrix_3 * Matrix::IDENTITY, matrix_3);
        assert_eq!(matrix_4 * Matrix::IDENTITY, matrix_4);
    }

    #[test]
    fn transpose_matrix() {
        let matrix_2 = Matrix::new([[1.0, 2.0], [5.5, 6.5]]);
        let matrix_2_transposed = Matrix::new([[1.0, 5.5], [2.0, 6.5]]);
        let matrix_3 = Matrix::new([[0.0, 9.0, 3.0], [9.0, 8.0, 0.0], [1.0, 8.0, 5.0]]);
        let matrix_3_transposed = Matrix::new([[0.0, 9.0, 1.0], [9.0, 8.0, 8.0], [3.0, 0.0, 5.0]]);
        let matrix_4 = Matrix::new([
            [0.0, 9.0, 3.0, 0.0],
            [9.0, 8.0, 0.0, 8.0],
            [1.0, 8.0, 5.0, 3.0],
            [0.0, 0.0, 5.0, 8.0],
        ]);
        let matrix_4_transposed = Matrix::new([
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
        assert_eq!(Matrix::<2>::IDENTITY.transpose(), Matrix::IDENTITY);
        assert_eq!(Matrix::<3>::IDENTITY.transpose(), Matrix::IDENTITY);
        assert_eq!(Matrix::<4>::IDENTITY.transpose(), Matrix::IDENTITY);
    }

    #[test]
    fn matrix_determinant_2x2() {
        let matrix = Matrix::new([[1.0, 5.0], [-3.0, 2.0]]);
        assert_eq!(matrix.determinant(), 17.0);
    }

    #[test]
    fn matrix_determinant_3x3() {
        let matrix = Matrix::new([[1.0, 2.0, 6.0], [-5.0, 8.0, -4.0], [2.0, 6.0, 4.0]]);
        assert_eq!(matrix.cofactor(0, 0), 56.0);
        assert_eq!(matrix.cofactor(0, 1), 12.0);
        assert_eq!(matrix.cofactor(0, 2), -46.0);
        assert_eq!(matrix.determinant(), -196.0);
    }

    #[test]
    fn matrix_determinant_4x4() {
        let matrix = Matrix::new([
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
        let matrix_1 = Matrix::new([[1.0, 5.0, 0.], [-3.0, 2.0, 7.0], [0.0, 6.0, -3.0]]);
        let matrix_2 = Matrix::new([[-3.0, 2.0], [0.0, 6.0]]);
        assert_eq!(matrix_1.submatrix(0, 2), matrix_2);
    }

    #[test]
    fn submatrix_4x4() {
        let matrix_1 = Matrix::new([
            [-6.0, 1.0, 1.0, 6.0],
            [-8.0, 5.0, 8.0, 6.0],
            [-1.0, 0.0, 8.0, 2.],
            [-7.0, 1.0, -1.0, 1.0],
        ]);
        let matrix_2 = Matrix::new([[-6.0, 1.0, 6.0], [-8.0, 8.0, 6.0], [-7.0, -1.0, 1.0]]);
        assert_eq!(matrix_1.submatrix(2, 1), matrix_2);
    }

    #[test]
    fn matrix_minor_and_cofactor() {
        let matrix = Matrix::new([[3.0, 5.0, 0.0], [2.0, -1.0, -7.0], [6.0, -1.0, 5.0]]);
        assert_eq!(matrix.minor(0, 0), -12.0);
        assert_eq!(matrix.cofactor(0, 0), -12.0);
        assert_eq!(matrix.minor(1, 0), 25.0);
        assert_eq!(matrix.cofactor(1, 0), -25.0);
    }

    #[test]
    fn matrix_inverse() {
        let matrix_1 = Matrix::new([
            [-5.0, 2.0, 6.0, -8.0],
            [1.0, -5.0, 1.0, 8.0],
            [7.0, 7.0, -6.0, -7.0],
            [1.0, -3.0, 7.0, 4.0],
        ]);
        let matrix_2 = Matrix::new([
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
        let matrix_3 = Matrix::new([
            [9.0, 3.0, 0.0, 9.0],
            [-5.0, -2.0, 6.0, -3.0],
            [-4.0, 9.0, 6.0, 4.0],
            [-7.0, 6.0, 6.0, 2.0],
        ]);
        let matrix_4 = Matrix::new([
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
        assert_eq!(matrix_5[3][2], -160.0 / 532.0);
        assert_eq!(matrix_1.cofactor(3, 2), 105.0);
        assert_eq!(matrix_5[2][3], 105.0 / 532.0);
        assert_eq!(matrix_5, matrix_2);
        assert_eq!(matrix_3.inverse(), matrix_4);
    }

    #[test]
    fn matrix_product_mul_by_inverse() {
        let matrix_1 = Matrix::new([
            [3.0, -9.0, 7.0, 3.0],
            [3.0, -8.0, 2.0, -9.0],
            [-4.0, 4.0, 4.0, 1.0],
            [-6.0, 5.0, -1.0, 1.0],
        ]);
        let matrix_2 = Matrix::new([
            [8.0, 2.0, 2.0, 2.0],
            [3.0, -1.0, 7.0, 0.0],
            [7.0, 0.0, 5.0, 4.0],
            [6.0, -2.0, 0.0, 5.0],
        ]);
        let matrix_3 = matrix_1 * matrix_2;
        assert_eq!(matrix_3 * matrix_2.inverse(), matrix_1);
    }
}
