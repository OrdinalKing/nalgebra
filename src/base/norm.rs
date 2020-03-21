use num::Zero;
use std::ops::Neg;

use crate::allocator::Allocator;
use crate::base::{DefaultAllocator, Dim, Matrix, MatrixMN, Normed};
use crate::constraint::{SameNumberOfColumns, SameNumberOfRows, ShapeConstraint};
use crate::storage::{Storage, StorageMut};
use crate::{ComplexField, Scalar, SimdComplexField, Unit};
use simba::scalar::ClosedNeg;
use simba::simd::{SimdOption, SimdPartialOrd};

// FIXME: this should be be a trait on alga?
/// A trait for abstract matrix norms.
///
/// This may be moved to the alga crate in the future.
pub trait Norm<N: SimdComplexField> {
    /// Apply this norm to the given matrix.
    fn norm<R, C, S>(&self, m: &Matrix<N, R, C, S>) -> N::SimdRealField
    where
        R: Dim,
        C: Dim,
        S: Storage<N, R, C>;
    /// Use the metric induced by this norm to compute the metric distance between the two given matrices.
    fn metric_distance<R1, C1, S1, R2, C2, S2>(
        &self,
        m1: &Matrix<N, R1, C1, S1>,
        m2: &Matrix<N, R2, C2, S2>,
    ) -> N::SimdRealField
    where
        R1: Dim,
        C1: Dim,
        S1: Storage<N, R1, C1>,
        R2: Dim,
        C2: Dim,
        S2: Storage<N, R2, C2>,
        ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2>;
}

/// Euclidean norm.
pub struct EuclideanNorm;
/// Lp norm.
pub struct LpNorm(pub i32);
/// L-infinite norm aka. Chebytchev norm aka. uniform norm aka. suppremum norm.
pub struct UniformNorm;

impl<N: SimdComplexField> Norm<N> for EuclideanNorm {
    #[inline]
    fn norm<R, C, S>(&self, m: &Matrix<N, R, C, S>) -> N::SimdRealField
    where
        R: Dim,
        C: Dim,
        S: Storage<N, R, C>,
    {
        m.norm_squared().simd_sqrt()
    }

    #[inline]
    fn metric_distance<R1, C1, S1, R2, C2, S2>(
        &self,
        m1: &Matrix<N, R1, C1, S1>,
        m2: &Matrix<N, R2, C2, S2>,
    ) -> N::SimdRealField
    where
        R1: Dim,
        C1: Dim,
        S1: Storage<N, R1, C1>,
        R2: Dim,
        C2: Dim,
        S2: Storage<N, R2, C2>,
        ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2>,
    {
        m1.zip_fold(m2, N::SimdRealField::zero(), |acc, a, b| {
            let diff = a - b;
            acc + diff.simd_modulus_squared()
        })
        .simd_sqrt()
    }
}

impl<N: SimdComplexField> Norm<N> for LpNorm {
    #[inline]
    fn norm<R, C, S>(&self, m: &Matrix<N, R, C, S>) -> N::SimdRealField
    where
        R: Dim,
        C: Dim,
        S: Storage<N, R, C>,
    {
        m.fold(N::SimdRealField::zero(), |a, b| {
            a + b.simd_modulus().simd_powi(self.0)
        })
        .simd_powf(crate::convert(1.0 / (self.0 as f64)))
    }

    #[inline]
    fn metric_distance<R1, C1, S1, R2, C2, S2>(
        &self,
        m1: &Matrix<N, R1, C1, S1>,
        m2: &Matrix<N, R2, C2, S2>,
    ) -> N::SimdRealField
    where
        R1: Dim,
        C1: Dim,
        S1: Storage<N, R1, C1>,
        R2: Dim,
        C2: Dim,
        S2: Storage<N, R2, C2>,
        ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2>,
    {
        m1.zip_fold(m2, N::SimdRealField::zero(), |acc, a, b| {
            let diff = a - b;
            acc + diff.simd_modulus().simd_powi(self.0)
        })
        .simd_powf(crate::convert(1.0 / (self.0 as f64)))
    }
}

impl<N: SimdComplexField> Norm<N> for UniformNorm {
    #[inline]
    fn norm<R, C, S>(&self, m: &Matrix<N, R, C, S>) -> N::SimdRealField
    where
        R: Dim,
        C: Dim,
        S: Storage<N, R, C>,
    {
        // NOTE: we don't use `m.amax()` here because for the complex
        // numbers this will return the max norm1 instead of the modulus.
        m.fold(N::SimdRealField::zero(), |acc, a| {
            acc.simd_max(a.simd_modulus())
        })
    }

    #[inline]
    fn metric_distance<R1, C1, S1, R2, C2, S2>(
        &self,
        m1: &Matrix<N, R1, C1, S1>,
        m2: &Matrix<N, R2, C2, S2>,
    ) -> N::SimdRealField
    where
        R1: Dim,
        C1: Dim,
        S1: Storage<N, R1, C1>,
        R2: Dim,
        C2: Dim,
        S2: Storage<N, R2, C2>,
        ShapeConstraint: SameNumberOfRows<R1, R2> + SameNumberOfColumns<C1, C2>,
    {
        m1.zip_fold(m2, N::SimdRealField::zero(), |acc, a, b| {
            let val = (a - b).simd_modulus();
            acc.simd_max(val)
        })
    }
}

impl<N: SimdComplexField, R: Dim, C: Dim, S: Storage<N, R, C>> Matrix<N, R, C, S> {
    /// The squared L2 norm of this vector.
    #[inline]
    pub fn norm_squared(&self) -> N::SimdRealField {
        let mut res = N::SimdRealField::zero();

        for i in 0..self.ncols() {
            let col = self.column(i);
            res += col.dotc(&col).simd_real()
        }

        res
    }

    /// The L2 norm of this matrix.
    ///
    /// Use `.apply_norm` to apply a custom norm.
    #[inline]
    pub fn norm(&self) -> N::SimdRealField {
        self.norm_squared().simd_sqrt()
    }

    /// Compute the distance between `self` and `rhs` using the metric induced by the euclidean norm.
    ///
    /// Use `.apply_metric_distance` to apply a custom norm.
    #[inline]
    pub fn metric_distance<R2, C2, S2>(&self, rhs: &Matrix<N, R2, C2, S2>) -> N::SimdRealField
    where
        R2: Dim,
        C2: Dim,
        S2: Storage<N, R2, C2>,
        ShapeConstraint: SameNumberOfRows<R, R2> + SameNumberOfColumns<C, C2>,
    {
        self.apply_metric_distance(rhs, &EuclideanNorm)
    }

    /// Uses the given `norm` to compute the norm of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Vector3, UniformNorm, LpNorm, EuclideanNorm};
    ///
    /// let v = Vector3::new(1.0, 2.0, 3.0);
    /// assert_eq!(v.apply_norm(&UniformNorm), 3.0);
    /// assert_eq!(v.apply_norm(&LpNorm(1)), 6.0);
    /// assert_eq!(v.apply_norm(&EuclideanNorm), v.norm());
    /// ```
    #[inline]
    pub fn apply_norm(&self, norm: &impl Norm<N>) -> N::SimdRealField {
        norm.norm(self)
    }

    /// Uses the metric induced by the given `norm` to compute the metric distance between `self` and `rhs`.
    ///
    /// # Example
    ///
    /// ```
    /// # use nalgebra::{Vector3, UniformNorm, LpNorm, EuclideanNorm};
    ///
    /// let v1 = Vector3::new(1.0, 2.0, 3.0);
    /// let v2 = Vector3::new(10.0, 20.0, 30.0);
    ///
    /// assert_eq!(v1.apply_metric_distance(&v2, &UniformNorm), 27.0);
    /// assert_eq!(v1.apply_metric_distance(&v2, &LpNorm(1)), 27.0 + 18.0 + 9.0);
    /// assert_eq!(v1.apply_metric_distance(&v2, &EuclideanNorm), (v1 - v2).norm());
    /// ```
    #[inline]
    pub fn apply_metric_distance<R2, C2, S2>(
        &self,
        rhs: &Matrix<N, R2, C2, S2>,
        norm: &impl Norm<N>,
    ) -> N::SimdRealField
    where
        R2: Dim,
        C2: Dim,
        S2: Storage<N, R2, C2>,
        ShapeConstraint: SameNumberOfRows<R, R2> + SameNumberOfColumns<C, C2>,
    {
        norm.metric_distance(self, rhs)
    }

    /// A synonym for the norm of this matrix.
    ///
    /// Aka the length.
    ///
    /// This function is simply implemented as a call to `norm()`
    #[inline]
    pub fn magnitude(&self) -> N::SimdRealField {
        self.norm()
    }

    /// A synonym for the squared norm of this matrix.
    ///
    /// Aka the squared length.
    ///
    /// This function is simply implemented as a call to `norm_squared()`
    #[inline]
    pub fn magnitude_squared(&self) -> N::SimdRealField {
        self.norm_squared()
    }

    /// Sets the magnitude of this vector.
    #[inline]
    pub fn set_magnitude(&mut self, magnitude: N::SimdRealField)
    where S: StorageMut<N, R, C> {
        let n = self.norm();
        self.scale_mut(magnitude / n)
    }

    /// Returns a normalized version of this matrix.
    #[inline]
    #[must_use = "Did you mean to use normalize_mut()?"]
    pub fn normalize(&self) -> MatrixMN<N, R, C>
    where DefaultAllocator: Allocator<N, R, C> {
        self.unscale(self.norm())
    }

    /// The Lp norm of this matrix.
    #[inline]
    pub fn lp_norm(&self, p: i32) -> N::SimdRealField {
        self.apply_norm(&LpNorm(p))
    }

    #[inline]
    #[must_use = "Did you mean to use simd_try_normalize_mut()?"]
    pub fn simd_try_normalize(&self, min_norm: N::SimdRealField) -> SimdOption<MatrixMN<N, R, C>>
    where
        N::Element: Scalar,
        DefaultAllocator: Allocator<N, R, C> + Allocator<N::Element, R, C>,
    {
        let n = self.norm();
        let le = n.simd_le(min_norm);
        let val = self.unscale(n);
        SimdOption::new(val, le)
    }
}

impl<N: ComplexField, R: Dim, C: Dim, S: Storage<N, R, C>> Matrix<N, R, C, S> {
    /// Sets the magnitude of this vector unless it is smaller than `min_magnitude`.
    ///
    /// If `self.magnitude()` is smaller than `min_magnitude`, it will be left unchanged.
    /// Otherwise this is equivalent to: `*self = self.normalize() * magnitude.
    #[inline]
    pub fn try_set_magnitude(&mut self, magnitude: N::RealField, min_magnitude: N::RealField)
    where S: StorageMut<N, R, C> {
        let n = self.norm();

        if n >= min_magnitude {
            self.scale_mut(magnitude / n)
        }
    }

    /// Returns a normalized version of this matrix unless its norm as smaller or equal to `eps`.
    #[inline]
    #[must_use = "Did you mean to use try_normalize_mut()?"]
    pub fn try_normalize(&self, min_norm: N::RealField) -> Option<MatrixMN<N, R, C>>
    where DefaultAllocator: Allocator<N, R, C> {
        let n = self.norm();

        if n <= min_norm {
            None
        } else {
            Some(self.unscale(n))
        }
    }
}

impl<N: SimdComplexField, R: Dim, C: Dim, S: StorageMut<N, R, C>> Matrix<N, R, C, S> {
    /// Normalizes this matrix in-place and returns its norm.
    #[inline]
    pub fn normalize_mut(&mut self) -> N::SimdRealField {
        let n = self.norm();
        self.unscale_mut(n);

        n
    }

    #[inline]
    #[must_use = "Did you mean to use simd_try_normalize_mut()?"]
    pub fn simd_try_normalize_mut(
        &mut self,
        min_norm: N::SimdRealField,
    ) -> SimdOption<N::SimdRealField>
    where
        N::Element: Scalar,
        DefaultAllocator: Allocator<N, R, C> + Allocator<N::Element, R, C>,
    {
        let n = self.norm();
        let le = n.simd_le(min_norm);
        self.apply(|e| e.simd_unscale(n).select(le, e));
        SimdOption::new(n, le)
    }
}

impl<N: ComplexField, R: Dim, C: Dim, S: StorageMut<N, R, C>> Matrix<N, R, C, S> {
    /// Normalizes this matrix in-place or does nothing if its norm is smaller or equal to `eps`.
    ///
    /// If the normalization succeeded, returns the old norm of this matrix.
    #[inline]
    pub fn try_normalize_mut(&mut self, min_norm: N::RealField) -> Option<N::RealField> {
        let n = self.norm();

        if n <= min_norm {
            None
        } else {
            self.unscale_mut(n);
            Some(n)
        }
    }
}

impl<N: SimdComplexField, R: Dim, C: Dim> Normed for MatrixMN<N, R, C>
where DefaultAllocator: Allocator<N, R, C>
{
    type Norm = N::SimdRealField;

    #[inline]
    fn norm(&self) -> N::SimdRealField {
        self.norm()
    }

    #[inline]
    fn norm_squared(&self) -> N::SimdRealField {
        self.norm_squared()
    }

    #[inline]
    fn scale_mut(&mut self, n: Self::Norm) {
        self.scale_mut(n)
    }

    #[inline]
    fn unscale_mut(&mut self, n: Self::Norm) {
        self.unscale_mut(n)
    }
}

impl<N: Scalar + ClosedNeg, R: Dim, C: Dim> Neg for Unit<MatrixMN<N, R, C>>
where DefaultAllocator: Allocator<N, R, C>
{
    type Output = Unit<MatrixMN<N, R, C>>;

    #[inline]
    fn neg(self) -> Self::Output {
        Unit::new_unchecked(-self.value)
    }
}
