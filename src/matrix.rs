//! Efficient multi-dimensional alternative to `Vec`.

// This module goes a bit beyond what I should need for this project because I'm having fun building it.
// Maybe it will be useful in the future?
#![allow(dead_code)]

use std::iter::{repeat_with, FromIterator};
use std::ops::{Index, IndexMut};

/// A multi-dimensional storage for data. More efficient, as a storage, than nested `Vec`s.
#[derive(Clone, Default, Debug)]
pub struct Matrix<T> {
    elements: Vec<T>,
    dimensions: Vec<usize>,
}

impl<T> Matrix<T> {
    fn offset(&self, index: &[usize]) -> usize {
        self.dimensions
            .iter()
            .zip(index)
            .fold(0, |offset, (len, index)| {
                assert!(index < len, "matrix index out of range");
                offset * len + index
            })
    }

    pub fn dimensions(&self) -> &[usize] {
        &self.dimensions
    }

    pub fn with_shape(mut self, dimensions: &[usize]) -> Self {
        self.reshape(dimensions);
        self
    }

    pub fn reshape(&mut self, dimensions: &[usize]) {
        assert_eq!(
            self.elements.len(),
            dimensions.iter().product(),
            "matrix cannot be reshaped with the wrong number of elements"
        );
        self.dimensions = dimensions.to_vec();
    }
}

impl<T> Matrix<T>
where
    T: Default,
{
    pub fn push_dimension_default(&mut self, dimension: usize) {
        assert!(
            dimension < self.dimensions.len(),
            "matrix dimension out of bounds"
        );
        let mut dimensions: Vec<_> = self.dimensions.iter().map(|dim| dim - 1).collect();
        let extend_size = self.dimensions[dimension + 1..].iter().product();
        'outer: loop {
            let offset = self.offset(&dimensions);
            self.elements.splice(
                (offset + 1)..(offset + 1),
                repeat_with(T::default).take(extend_size),
            );
            let mut next_dim = dimension.checked_sub(1);
            loop {
                match next_dim {
                    Some(dim) => match dimensions[dim].checked_sub(1) {
                        Some(decr) => {
                            dimensions[dim] = decr;
                            break;
                        }
                        None => {
                            dimensions[dim] = self.dimensions[dim] - 1;
                            next_dim = dim.checked_sub(1);
                        }
                    },
                    None => break 'outer,
                }
            }
        }
        self.dimensions[dimension] += 1;
    }
}

impl<T> Index<&[usize]> for Matrix<T> {
    type Output = T;

    fn index(&self, index: &[usize]) -> &T {
        assert_eq!(
            index.len(),
            self.dimensions.len(),
            "incorrect matrix index dimension"
        );
        let offset = self.offset(index);
        &self.elements[offset]
    }
}

impl<T> IndexMut<&[usize]> for Matrix<T> {
    fn index_mut(&mut self, index: &[usize]) -> &mut T {
        assert_eq!(
            index.len(),
            self.dimensions.len(),
            "incorrect matrix index dimension"
        );
        let offset = self.offset(index);
        &mut self.elements[offset]
    }
}

impl<T> From<Vec<T>> for Matrix<T> {
    fn from(elements: Vec<T>) -> Self {
        Self {
            dimensions: vec![elements.len()],
            elements,
        }
    }
}

impl<T> Into<Vec<T>> for Matrix<T> {
    fn into(self) -> Vec<T> {
        self.elements
    }
}

impl<T> FromIterator<Vec<T>> for Matrix<T>
where
    T: Default,
{
    fn from_iter<I: IntoIterator<Item = Vec<T>>>(iter: I) -> Self {
        let rows: Vec<Vec<T>> = iter.into_iter().collect();
        let max_width = rows.iter().map(|record| record.len()).max().unwrap_or(1);
        let elements: Vec<T> = rows
            .into_iter()
            .flat_map(|row| {
                let len = row.len();
                row.into_iter()
                    .chain(repeat_with(T::default).take(max_width - len))
            })
            .collect();
        Matrix {
            dimensions: vec![elements.len() / max_width, max_width],
            elements,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matrix_index() {
        let matrix = Matrix::from_iter(vec![vec![1, 2, 3], vec![1, 2, 3], vec![1, 2, 3]]);
        assert_eq!(3, matrix[&[0, 2]]);
        assert_eq!(2, matrix[&[1, 1]]);
        assert_eq!(1, matrix[&[2, 0]]);
    }

    #[test]
    fn matrix_push_dimension_default_0() {
        let mut matrix = Matrix::from_iter(vec![vec![1, 2, 3], vec![1, 2, 3], vec![1, 2, 3]]);
        matrix.push_dimension_default(0);
        assert_eq!(matrix.dimensions, vec![4, 3]);
        assert_eq!(matrix.elements, vec![1, 2, 3, 1, 2, 3, 1, 2, 3, 0, 0, 0]);
    }

    #[test]
    fn matrix_push_dimension_default_1() {
        let mut matrix = Matrix::from_iter(vec![vec![1, 2, 3], vec![1, 2, 3], vec![1, 2, 3]]);
        matrix.push_dimension_default(1);
        assert_eq!(matrix.dimensions, vec![3, 4]);
        assert_eq!(matrix.elements, vec![1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0]);
    }

    #[test]
    fn matrix_push_dimension_default_2() {
        let mut matrix = Matrix::from(vec![1, 2, 3, 4, 1, 2, 3, 4]).with_shape(&[2, 2, 2]);
        matrix.push_dimension_default(1);
        assert_eq!(matrix.dimensions, vec![2, 3, 2]);
        assert_eq!(matrix.elements, vec![1, 2, 3, 4, 0, 0, 1, 2, 3, 4, 0, 0]);
    }

    #[test]
    fn matrix_push_dimension_default_3() {
        let mut matrix = Matrix::from(vec![1, 2, 3, 4, 1, 2, 3, 4]).with_shape(&[2, 2, 2]);
        matrix.push_dimension_default(2);
        assert_eq!(matrix.dimensions, vec![2, 2, 3]);
        assert_eq!(matrix.elements, vec![1, 2, 0, 3, 4, 0, 1, 2, 0, 3, 4, 0]);
    }
}
