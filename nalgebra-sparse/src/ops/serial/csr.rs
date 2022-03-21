use crate::csr::CsrMatrix;
use crate::ops::serial::cs::{
    spadd_cs_prealloc, spmm_cs_dense, spmm_cs_prealloc_checked, spmm_cs_prealloc_unchecked,
};
use crate::ops::serial::OperationError;
use crate::ops::Op;
use nalgebra::{ClosedAdd, ClosedMul, DMatrixSlice, DMatrixSliceMut, Scalar};
use num_traits::{One, Zero};
use std::borrow::Cow;

/// Sparse-dense matrix-matrix multiplication `C <- beta * C + alpha * op(A) * op(B)`.
pub fn spmm_csr_dense<'a, T>(
    beta: T,
    c: impl Into<DMatrixSliceMut<'a, T>>,
    alpha: T,
    a: Op<&CsrMatrix<T>>,
    b: Op<impl Into<DMatrixSlice<'a, T>>>,
) where
    T: Scalar + ClosedAdd + ClosedMul + Zero + One,
{
    let b = b.convert();
    spmm_csr_dense_(beta, c.into(), alpha, a, b)
}

fn spmm_csr_dense_<T>(
    beta: T,
    c: DMatrixSliceMut<'_, T>,
    alpha: T,
    a: Op<&CsrMatrix<T>>,
    b: Op<DMatrixSlice<'_, T>>,
) where
    T: Scalar + ClosedAdd + ClosedMul + Zero + One,
{
    assert_compatible_spmm_dims!(c, a, b);
    spmm_cs_dense(beta, c, alpha, a.map_same_op(|a| &a.cs), b)
}

/// Sparse matrix addition `C <- beta * C + alpha * op(A)`.
///
/// # Errors
///
/// If the pattern of `c` does not accommodate all the non-zero entries in `a`, an error is
/// returned.
///
/// # Panics
///
/// Panics if the dimensions of the matrices involved are not compatible with the expression.
pub fn spadd_csr_prealloc<T>(
    beta: T,
    c: &mut CsrMatrix<T>,
    alpha: T,
    a: Op<&CsrMatrix<T>>,
) -> Result<(), OperationError>
where
    T: Scalar + ClosedAdd + ClosedMul + Zero + One,
{
    assert_compatible_spadd_dims!(c, a);
    spadd_cs_prealloc(beta, &mut c.cs, alpha, a.map_same_op(|a| &a.cs))
}

/// Sparse-sparse matrix multiplication, `C <- beta * C + alpha * op(A) * op(B)`.
///
/// # Errors
///
/// If the pattern of `C` is not able to hold the result of the operation, an error is returned.
///
/// # Panics
///
/// Panics if the dimensions of the matrices involved are not compatible with the expression.
pub fn spmm_csr_prealloc_checked<T>(
    beta: T,
    c: &mut CsrMatrix<T>,
    alpha: T,
    a: Op<&CsrMatrix<T>>,
    b: Op<&CsrMatrix<T>>,
) -> Result<(), OperationError>
where
    T: Scalar + ClosedAdd + ClosedMul + Zero + One,
{
    assert_compatible_spmm_dims!(c, a, b);

    use Op::NoOp;

    match (&a, &b) {
        (NoOp(ref a), NoOp(ref b)) => {
            spmm_cs_prealloc_checked(beta, &mut c.cs, alpha, &a.cs, &b.cs)
        }
        _ => do_transposes(beta, c, alpha, a, b, spmm_csr_prealloc_checked),
    }
}

/// Faster sparse-sparse matrix multiplication, `C <- beta * C + alpha * op(A) * op(B)`.
/// This will not return an error even if the patterns don't match.
/// Should be used for situations where pattern creation immediately preceeds multiplication.
///
/// Panics if the dimensions of the matrices involved are not compatible with the expression.
pub(crate) fn spmm_csr_prealloc_unchecked<T>(
    beta: T,
    c: &mut CsrMatrix<T>,
    alpha: T,
    a: Op<&CsrMatrix<T>>,
    b: Op<&CsrMatrix<T>>,
) -> Result<(), OperationError>
where
    T: Scalar + ClosedAdd + ClosedMul + Zero + One,
{
    assert_compatible_spmm_dims!(c, a, b);

    use Op::NoOp;

    match (&a, &b) {
        (NoOp(ref a), NoOp(ref b)) => {
            spmm_cs_prealloc_unchecked(beta, &mut c.cs, alpha, &a.cs, &b.cs)
        }
        _ => do_transposes(beta, c, alpha, a, b, spmm_csr_prealloc_unchecked),
    }
}

fn do_transposes<T, F>(
    beta: T,
    c: &mut CsrMatrix<T>,
    alpha: T,
    a: Op<&CsrMatrix<T>>,
    b: Op<&CsrMatrix<T>>,
    caller: F,
) -> Result<(), OperationError>
where
    T: Scalar + ClosedAdd + ClosedMul + Zero + One,
    F: Fn(
        T,
        &mut CsrMatrix<T>,
        T,
        Op<&CsrMatrix<T>>,
        Op<&CsrMatrix<T>>,
    ) -> Result<(), OperationError>,
{
    use Op::{NoOp, Transpose};

    // Currently we handle transposition by explicitly precomputing transposed matrices
    // and calling the operation again without transposition
    let a_ref: &CsrMatrix<T> = a.inner_ref();
    let b_ref: &CsrMatrix<T> = b.inner_ref();
    let (a, b) = {
        use Cow::*;
        match (&a, &b) {
            (NoOp(_), NoOp(_)) => unreachable!(),
            (Transpose(ref a), NoOp(_)) => (Owned(a.transpose()), Borrowed(b_ref)),
            (NoOp(_), Transpose(ref b)) => (Borrowed(a_ref), Owned(b.transpose())),
            (Transpose(ref a), Transpose(ref b)) => (Owned(a.transpose()), Owned(b.transpose())),
        }
    };
    caller(beta, c, alpha, NoOp(a.as_ref()), NoOp(b.as_ref()))
}
