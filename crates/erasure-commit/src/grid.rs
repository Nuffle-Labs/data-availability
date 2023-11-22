use nalgebra::{DMatrix, Scalar};

/// A Grid of data, also known as a perfect matrix
pub struct Grid<T> {
    pub width_length: usize,
    pub inner: DMatrix<T>,
}

// TODO[optimisation, reusability]: Remove tight coupling to Scalar
impl<T: Scalar> Grid<T> {
    /// Create a new grid, filling it with nullifiers to expand the data to a perfect
    /// square.
    pub fn new(mut data: Vec<T>, nullifier: &T) -> Self {
        // Number of field elements created from data
        let sq_root = Grid::square_dimensions(&data);

        let grow_len = sq_root * sq_root;
        if data.len() as u64 != grow_len {
            println!("Expanding data from {} to {}", data.len(), grow_len);
            data.resize(grow_len as usize, nullifier.clone());
        }

        let matrix = DMatrix::from_vec(sq_root as usize, sq_root as usize, data);
        Grid {
            width_length: sq_root as usize,
            inner: matrix,
        }
    }

    /// Gather the grid dimensions required to create a perfect square
    pub fn square_dimensions(data: &Vec<T>) -> u64 {
        let mut sq_root = (data.len() as f64).sqrt().ceil() as u64;
        if sq_root % 2 != 0 {
            sq_root += 1;
        }
        println!("sq_root: {sq_root}, data len: {}", data.len());
        sq_root
    }

    /// The inner matrix takes ownership in a builder fashion, this allows us to
    /// a more ergonomic builder.
    pub fn update(&mut self, matrix: DMatrix<T>) {
        self.inner = matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn squared_data(capacity: usize) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(capacity);
        for i in 1..=data.capacity() {
            data.push(i as u8);
        }
        data
    }

    #[test]
    fn test_square() {
        assert_eq!(4, Grid::square_dimensions(&squared_data(16)));
        assert_eq!(60, Grid::square_dimensions(&squared_data(3600)));
    }

    #[test]
    fn test_matrix_arrangement() {
        let matrix = Grid::new(squared_data(16), &1);
        let matrix = matrix.inner.remove_row(3);
        assert_eq!(matrix.len(), 12);
        let matrix = matrix.remove_column(3);
        assert_eq!(matrix.len(), 9);
    }
}
