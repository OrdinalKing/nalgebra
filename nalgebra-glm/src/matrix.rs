use na::{Scalar, Real, DefaultAllocator};

use traits::{Alloc, Dimension, Number};
use aliases::{Mat, Vec};

/// The determinant of the matrix `m`.
pub fn determinant<N: Real, D: Dimension>(m: &Mat<N, D, D>) -> N
    where DefaultAllocator: Alloc<N, D, D> {
    m.determinant()
}

/// The inverse of the matrix `m`.
pub fn inverse<N: Real, D: Dimension>(m: &Mat<N, D, D>) -> Mat<N, D, D>
    where DefaultAllocator: Alloc<N, D, D> {
    m.clone().try_inverse().unwrap_or(Mat::<N, D, D>::zeros())
}

/// Componentwise multiplication of two matrices.
pub fn matrix_comp_mult<N: Number, R: Dimension, C: Dimension>(x: &Mat<N, R, C>, y: &Mat<N, R, C>) -> Mat<N, R, C>
    where DefaultAllocator: Alloc<N, R, C> {
    x.component_mul(y)
}

/// Treats the first parameter `c` as a column vector and the second parameter `r` as a row vector and does a linear algebraic matrix multiply `c * r`.
pub fn outer_product<N: Number, R: Dimension, C: Dimension>(c: &Vec<N, R>, r: &Vec<N, C>) -> Mat<N, R, C>
    where DefaultAllocator: Alloc<N, R, C> {
    c * r.transpose()
}

/// The transpose of the matrix `m`.
pub fn transpose<N: Scalar, R: Dimension, C: Dimension>(x: &Mat<N, R, C>) -> Mat<N, C, R>
    where DefaultAllocator: Alloc<N, R, C> {
    x.transpose()
}