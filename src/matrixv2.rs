// use std::ops::{Index, Mul};
// use crate::{Tuple, TupleTrait};
//
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct Matrixv2<const SIDE_LENGTH: usize> {
//     pub elements: [[f64; SIDE_LENGTH]; SIDE_LENGTH],
// }
//
// impl<const SIDE_LENGTH: usize> Matrixv2<SIDE_LENGTH> {
//     pub fn new(elements: [[f64; SIDE_LENGTH]; SIDE_LENGTH]) -> Matrixv2<SIDE_LENGTH> {
//         return Matrixv2 { elements };
//     }
//
//     pub fn new_empty() -> Self {
//         return Matrixv2 { elements: [[0.0; SIDE_LENGTH]; SIDE_LENGTH] };
//     }
//
//     pub fn get_index(&self, row: usize, column: usize) -> f64 {
//         return self.elements[row][column];
//     }
//
//     pub fn set_index(&mut self, row: usize, column: usize, value: f64) {
//         self.elements[row][column] = value;
//     }
//
//     pub fn identity() -> Matrixv2<SIDE_LENGTH> {
//         let mut matrix = Matrixv2::<SIDE_LENGTH>::new_empty();
//         for index in 0..SIDE_LENGTH {
//             matrix.set_index(index, index, 1.0);
//         }
//         return matrix;
//     }
//
//     pub fn transpose(&self) -> Matrixv2<SIDE_LENGTH> {
//         let mut matrix = Matrixv2::<SIDE_LENGTH>::new_empty();
//         for row in 0..SIDE_LENGTH {
//             for column in 0..SIDE_LENGTH {
//                 matrix.set_index(row, column, self.get_index(column, row));
//             }
//         }
//         return matrix;
//     }
//
//     pub fn determinant(&self) -> f64 where [(); SIDE_LENGTH - 1]:, [(); SIDE_LENGTH - 1 - 1]: {
//         let mut determinant = 0.0;
//         match SIDE_LENGTH {
//             _ if SIDE_LENGTH < 2 => panic!("Invalid array side length"),
//             _ if SIDE_LENGTH == 2 => determinant = self.elements[0][0] * self.elements[1][1] - self.elements[0][1] * self.elements[1][0],
//             _ => {
//                 for column in 0..SIDE_LENGTH {
//                     determinant += self.get_index(0, column) * self.cofactor(0, column);
//                 }
//             }
//         }
//         return determinant;
//     }
//
//     fn submatrix(&self, row: usize, column: usize) -> Matrixv2<{ SIDE_LENGTH - 1 }> where [(); SIDE_LENGTH - 1]:, [(); SIDE_LENGTH - 1 - 1]: {
//         let mut matrix = Matrixv2::<{ SIDE_LENGTH - 1 }>::new_empty();
//         for row_index in 0..SIDE_LENGTH {
//             if row_index != row {
//                 for column_index in 0..SIDE_LENGTH {
//                     if column_index != column {
//                         matrix.set_index(row_index, column_index, self.elements[row_index][column_index]);
//                     }
//                 }
//             }
//         }
//         return matrix;
//     }
//
//     pub fn minor(&self, row: usize, column: usize) -> f64 where [(); SIDE_LENGTH - 1]:, [(); SIDE_LENGTH - 1 - 1]: {
//         return self.submatrix(row, column).determinant();
//     }
//
//     pub fn cofactor(&self, row: usize, column: usize) -> f64 where [(); SIDE_LENGTH - 1]:, [(); SIDE_LENGTH - 1 - 1]: {
//         let minor = self.minor(row, column);
//         return if (row + column) % 2 == 0 { minor } else { -minor };
//     }
//
//     pub fn revertible(&self) -> bool where [(); SIDE_LENGTH - 1]:, [(); SIDE_LENGTH - 1 - 1 ]: {
//         return self.determinant() != 0.0;
//     }
//
//     pub fn side_length(&self) -> usize {
//         return SIDE_LENGTH;
//     }
// }
//
// impl<const SIDE_LENGTH: usize> Default for Matrixv2<SIDE_LENGTH> {
//     fn default() -> Self {
//         let mut new_matrix = Matrixv2::<SIDE_LENGTH>::new_empty();
//         for i in 0..SIDE_LENGTH {
//             new_matrix.set_index(i, i, 1.0);
//         }
//         return new_matrix;
//     }
// }
//
// impl<const SIDE_LENGTH: usize> Index<usize> for Matrixv2<SIDE_LENGTH> {
//     type Output = [f64; SIDE_LENGTH];
//
//     fn index(&self, index: usize) -> &Self::Output {
//         return &self.elements[index];
//     }
// }
//
// impl<const SIDE_LENGTH: usize> Mul<Matrixv2<SIDE_LENGTH>> for Matrixv2<SIDE_LENGTH> {
//     type Output = Matrixv2<SIDE_LENGTH>;
//
//     fn mul(self, rhs: Matrixv2<SIDE_LENGTH>) -> Self::Output {
//         let mut result_matrix = Matrixv2::<SIDE_LENGTH>::new_empty();
//         for row in 0..SIDE_LENGTH  {
//             for column in 0..SIDE_LENGTH {
//                 let mut sum = 0.0;
//                 for i in 0..SIDE_LENGTH {
//                     sum += self.get_index(row, i)  * rhs.get_index(i, column);
//                 }
//                 result_matrix.set_index(row, column, sum);
//             }
//         }
//         return result_matrix;
//     }
// }
//
// impl<const SIDE_LENGTH: usize> Mul<Tuple> for Matrixv2<SIDE_LENGTH> {
//     type Output = Tuple;
//
//     fn mul(self, rhs: Tuple) -> Self::Output {
//         if SIDE_LENGTH != 4 {
//             panic!("Matrix must be 4x4 to be multiplied by a tuple");
//         }
//
//         let mut result = [0.0; 4];
//         let tuple_values = rhs.get_values();
//         for i in 0..4 {
//             let mut sum = 0.0;
//             for j in 0..4 {
//                 sum += self.get_index(i, j) * tuple_values[j];
//             }
//             result[i] = sum;
//         }
//         return Tuple::new(result[0], result[1], result[2], result[3]);
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::{Tuple, TupleTrait};
//     use super::*;
//
//     #[test]
//     fn matrix_indices() {
//         let matrix = Matrixv2::new([
//             [1.0, 2.0, 3.0, 4.0],
//             [5.5, 6.5, 7.5, 8.5],
//             [9.0, 10.0, 11.0, 12.0],
//             [13.5, 14.5, 15.5, 16.5],
//         ]);
//         assert_eq!(matrix.get_index(0, 0), 1.0);
//         assert_eq!(matrix.get_index(0, 3), 4.0);
//         assert_eq!(matrix.get_index(1, 0), 5.5);
//         assert_eq!(matrix.get_index(1, 2), 7.5);
//         assert_eq!(matrix.get_index(2, 2), 11.0);
//         assert_eq!(matrix.get_index(3, 0), 13.5);
//         assert_eq!(matrix.get_index(3, 2), 15.5);
//     }
//
//     #[test]
//     fn matrix_comparison() {
//         let matrix1 = Matrixv2::new([
//             [1.0, 2.0, 3.0, 4.0],
//             [5.0, 6.0, 7.0, 8.0],
//             [9.0, 8.0, 7.0, 6.0],
//             [5.0, 4.0, 3.0, 2.0],
//         ]);
//         let matrix2 = matrix1;
//         let matrix3 = Matrixv2::new([
//             [2.0, 3.0, 4.0, 5.0],
//             [6.0, 7.0, 8.0, 9.0],
//             [8.0, 7.0, 6.0, 5.0],
//             [4.0, 3.0, 2.0, 1.0],
//         ]);
//         assert_eq!(matrix1, matrix2);
//         assert_ne!(matrix2, matrix3);
//     }
//
//     #[test]
//     fn matrix_multiplication_by_matrix() {
//         let matrix1 = Matrixv2::new([
//             [1.0, 2.0, 3.0, 4.0],
//             [5.0, 6.0, 7.0, 8.0],
//             [9.0, 8.0, 7.0, 6.0],
//             [5.0, 4.0, 3.0, 2.0],
//         ]);
//         let matrix2 = Matrixv2::new([
//             [-2.0, 1.0, 2.0, 3.0],
//             [3.0, 2.0, 1.0, -1.0],
//             [4.0, 3.0, 6.0, 5.0],
//             [1.0, 2.0, 7.0, 8.0],
//         ]);
//         let result_matrix = Matrixv2::new([
//             [20.0, 22.0, 50.0, 48.0],
//             [44.0, 54.0, 114.0, 108.0],
//             [40.0, 58.0, 110.0, 102.0],
//             [16.0, 26.0, 46.0, 42.0],
//         ]);
//         assert_eq!(matrix1 * matrix2, result_matrix);
//     }
//
//     #[test]
//     fn matrix_multiplication_by_tuple() {
//         let matrix = Matrixv2::new([
//             [1.0, 2.0, 3.0, 4.0],
//             [2.0, 4.0, 4.0, 2.0],
//             [8.0, 6.0, 4.0, 1.0],
//             [0.0, 0.0, 0.0, 1.0],
//         ]);
//         let tuple = Tuple::new(1.0, 2.0, 3.0, 1.0);
//         assert_eq!(matrix * tuple, Tuple::new(18.0, 24.0, 33.0, 1.0));
//     }
//
//     #[test]
//     fn identity_matrix() {
//         let matrix1 = Matrixv2::new([
//             [0.0, 1.0, 2.0, 4.0],
//             [1.0, 2.0, 4.0, 8.0],
//             [2.0, 4.0, 8.0, 16.0],
//             [4.0, 8.0, 16.0, 32.0],
//         ]);
//         assert_eq!((matrix1 * Matrixv2::identity()), matrix1);
//     }
//
//     #[test]
//     fn transpose_matrix() {
//         let matrix1 = Matrixv2::new([
//             [0.0, 9.0, 3.0, 0.0],
//             [9.0, 8.0, 0.0, 8.0],
//             [1.0, 8.0, 5.0, 3.0],
//             [0.0, 0.0, 5.0, 8.0],
//         ]);
//         let matrix2 = Matrixv2::new([
//             [0.0, 9.0, 1.0, 0.0],
//             [9.0, 8.0, 8.0, 0.0],
//             [3.0, 0.0, 5.0, 5.0],
//             [0.0, 8.0, 3.0, 8.0],
//         ]);
//         assert_eq!(matrix1.transpose(), matrix2);
//     }
//
//     #[test]
//     fn transpose_identity_matrix() {
//         let matrix1 = Matrixv2::<4>::identity();
//         assert_eq!(matrix1.transpose(), Matrixv2::identity());
//     }
//
//     #[test]
//     fn matrix_determinant_2x2() {
//         let matrix1 = Matrixv2::new([
//             [1.0, 5.0],
//             [-3.0, 2.0],
//         ]);
//         assert_eq!(matrix1.determinant(), 17.0);
//     }
//
//         #[test]
//     fn matrix_determinant_3x3() {
//         let matrix1 = Matrixv2::new([
//             [1.0, 2.0, 6.0],
//             [-5.0, 8.0, -4.0],
//             [2.0, 6.0, 4.0],
//         ]);
//         assert_eq!(matrix1.cofactor(0, 0), 56.0);
//         assert_eq!(matrix1.cofactor(0, 1), 12.0);
//         assert_eq!(matrix1.cofactor(0, 2), -46.0);
//         assert_eq!(matrix1.determinant(), -196.0);
//     }
//
//     #[test]
//     fn matrix_determinant_4x4() {
//         let matrix1 = Matrixv2::new([
//             [-2.0, -8.0, 3.0, 5.0],
//             [-3.0, 1.0, 7.0, 3.0],
//             [1.0, 2.0, -9.0, 6.0],
//             [-6.0, 7.0, 7.0, -9.0],
//         ]);
//         assert_eq!(matrix1.cofactor(0, 0), 690.0);
//         assert_eq!(matrix1.cofactor(0, 1), 447.0);
//         assert_eq!(matrix1.cofactor(0, 2), 210.0);
//         assert_eq!(matrix1.cofactor(0, 3), 51.0);
//         assert_eq!(matrix1.determinant(), -4071.0);
//     }
// }
