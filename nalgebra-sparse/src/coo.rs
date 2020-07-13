use crate::SparseFormatError;
use nalgebra::{ClosedAdd, DMatrix, Scalar};
use num_traits::Zero;

/// A COO representation of a sparse matrix.
///
/// A COO matrix stores entries in coordinate-form, that is triplets `(i, j, v)`, where `i` and `j`
/// correspond to row and column indices of the entry, and `v` to the value of the entry.
/// With the rare exception of matrix-vector multiplication of certain extremely sparse matrices,
/// it is of limited use for standard matrix operations. Its main purpose is to facilitate
/// easy construction of other, more efficient matrix formats (such as CSR/COO), and the
/// conversion between different formats.
///
/// Representation
/// --------------
///
/// For given dimensions `nrows` and `ncols`, the matrix is represented by three same-length
/// arrays `row_indices`, `col_indices` and `values` that constitute the coordinate triplets
/// of the matrix. The indices must be in bounds, but *duplicate entries are explicitly allowed*.
/// Upon conversion to other formats, the duplicate entries may be summed together. See the
/// documentation for the respective conversion functions.
///
/// Example
/// -------
///
/// ```rust
/// # use nalgebra_sparse::CooMatrix;
/// // Create a zero matrix
/// let mut coo = CooMatrix::new(4, 4);
/// // Or initialize it with a set of triplets
/// coo = CooMatrix::try_from_triplets(4, 4, vec![1, 2], vec![0, 1], vec![3.0, 4.0]).unwrap();
///
/// // Push a single triplet
/// coo.push(2, 0, 1.0);
///
/// // TODO: Convert to CSR
/// ```
#[derive(Debug, Clone)]
pub struct CooMatrix<T> {
    nrows: usize,
    ncols: usize,
    row_indices: Vec<usize>,
    col_indices: Vec<usize>,
    values: Vec<T>,
}

impl<T> CooMatrix<T>
where
    T: Scalar,
{
    /// Construct a zero COO matrix of the given dimensions.
    ///
    /// Specifically, the collection of triplets - corresponding to explicitly stored entries -
    /// is empty, so that the matrix (implicitly) represented by the COO matrix consists of all
    /// zero entries.
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows,
            ncols,
            row_indices: Vec::new(),
            col_indices: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Try to construct a COO matrix from the given dimensions and a collection of
    /// (i, j, v) triplets.
    ///
    /// Returns an error if either row or column indices contain indices out of bounds,
    /// or if the data arrays do not all have the same length. Note that the COO format
    /// inherently supports duplicate entries.
    pub fn try_from_triplets(
        nrows: usize,
        ncols: usize,
        row_indices: Vec<usize>,
        col_indices: Vec<usize>,
        values: Vec<T>,
    ) -> Result<Self, SparseFormatError> {
        if row_indices.len() != col_indices.len() {
            return Err(SparseFormatError::InvalidStructure(
                Box::from("Number of row and col indices must be the same.")
            ));
        } else if col_indices.len() != values.len() {
            return Err(SparseFormatError::InvalidStructure(
                Box::from("Number of col indices and values must be the same.")
            ));
        }

        let row_indices_in_bounds = row_indices.iter().all(|i| *i < nrows);
        let col_indices_in_bounds = col_indices.iter().all(|j| *j < ncols);

        if !row_indices_in_bounds {
            Err(SparseFormatError::IndexOutOfBounds(Box::from(
                "Row index out of bounds.",
            )))
        } else if !col_indices_in_bounds {
            Err(SparseFormatError::IndexOutOfBounds(Box::from(
                "Col index out of bounds.",
            )))
        } else {
            Ok(Self {
                nrows,
                ncols,
                row_indices,
                col_indices,
                values,
            })
        }
    }

    /// An iterator over triplets (i, j, v).
    // TODO: Consider giving the iterator a concrete type instead of impl trait...?
    pub fn triplet_iter(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        self.row_indices
            .iter()
            .zip(&self.col_indices)
            .zip(&self.values)
            .map(|((i, j), v)| (*i, *j, v))
    }

    /// Push a single triplet to the matrix.
    ///
    /// This adds the value `v` to the `i`th row and `j`th column in the matrix.
    ///
    /// Panics
    /// ------
    ///
    /// Panics if `i` or `j` is out of bounds.
    #[inline(always)]
    pub fn push(&mut self, i: usize, j: usize, v: T) {
        assert!(i < self.nrows);
        assert!(j < self.ncols);
        self.row_indices.push(i);
        self.col_indices.push(j);
        self.values.push(v);
    }

    /// The number of rows in the matrix.
    #[inline(always)]
    pub fn nrows(&self) -> usize {
        self.nrows
    }

    /// The number of columns in the matrix.
    #[inline(always)]
    pub fn ncols(&self) -> usize {
        self.ncols
    }

    /// The row indices of the explicitly stored entries.
    pub fn row_indices(&self) -> &[usize] {
        &self.row_indices
    }

    /// The column indices of the explicitly stored entries.
    pub fn col_indices(&self) -> &[usize] {
        &self.col_indices
    }

    /// The values of the explicitly stored entries.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// Disassembles the matrix into individual triplet arrays.
    ///
    /// Examples
    /// --------
    ///
    /// ```
    /// # use nalgebra_sparse::CooMatrix;
    /// let row_indices = vec![0, 1];
    /// let col_indices = vec![1, 2];
    /// let values = vec![1.0, 2.0];
    /// let coo = CooMatrix::try_from_triplets(2, 3, row_indices, col_indices, values)
    ///     .unwrap();
    ///
    /// let (row_idx, col_idx, val) = coo.disassemble();
    /// assert_eq!(row_idx, vec![0, 1]);
    /// assert_eq!(col_idx, vec![1, 2]);
    /// assert_eq!(val, vec![1.0, 2.0]);
    /// ```
    pub fn disassemble(self) -> (Vec<usize>, Vec<usize>, Vec<T>) {
        (self.row_indices, self.col_indices, self.values)
    }

    /// Construct the dense representation of the COO matrix.
    ///
    /// Duplicate entries are summed together.
    pub fn to_dense(&self) -> DMatrix<T>
    where
        T: ClosedAdd + Zero,
    {
        let mut result = DMatrix::zeros(self.nrows, self.ncols);

        for (i, j, v) in self.triplet_iter() {
            result[(i, j)] += v.clone();
        }

        result
    }
}
