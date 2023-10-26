use nalgebra::{DMatrix, Scalar};

pub struct Grid<T> {
    pub width_length: usize,
    pub inner: DMatrix<T>,
}

impl<T: Scalar> Grid<T> {
    pub fn new(mut data: Vec<T>, nullifier: &T) -> Self {
        // Number of field elements created from data
        let sq_root = Grid::square_from_data(&data);

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

    pub fn square_from_data(data: &Vec<T>) -> u64 {
        let mut sq_root = (data.len() as f64).sqrt().ceil() as u64;
        if sq_root % 2 != 0 {
            sq_root += 1;
        }
        println!("sq_root: {sq_root}, data len: {}", data.len());
        sq_root
    }

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
        assert_eq!(4, Grid::square_from_data(&squared_data(16)));
        assert_eq!(60, Grid::square_from_data(&squared_data(3600)));
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
