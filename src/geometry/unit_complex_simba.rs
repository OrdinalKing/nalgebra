use num_complex::Complex;
use simba::simd::SimdValue;
use std::ops::Deref;

use crate::base::{Scalar, Unit};
use crate::geometry::UnitComplex;
use crate::RealField;

impl<N: RealField> SimdValue for UnitComplex<N>
where N::Element: Scalar
{
    type Element = UnitComplex<N::Element>;
    type SimdBool = N::SimdBool;

    #[inline]
    fn lanes() -> usize {
        N::lanes()
    }

    #[inline]
    fn splat(val: Self::Element) -> Self {
        Unit::new_unchecked(Complex::splat(val.into_inner()))
    }

    #[inline]
    fn extract(&self, i: usize) -> Self::Element {
        Unit::new_unchecked(self.deref().extract(i))
    }

    #[inline]
    unsafe fn extract_unchecked(&self, i: usize) -> Self::Element {
        Unit::new_unchecked(self.deref().extract_unchecked(i))
    }

    #[inline]
    fn replace(&mut self, i: usize, val: Self::Element) {
        self.as_mut_unchecked().replace(i, val.into_inner())
    }

    #[inline]
    unsafe fn replace_unchecked(&mut self, i: usize, val: Self::Element) {
        self.as_mut_unchecked()
            .replace_unchecked(i, val.into_inner())
    }

    #[inline]
    fn select(self, cond: Self::SimdBool, other: Self) -> Self {
        Unit::new_unchecked(self.into_inner().select(cond, other.into_inner()))
    }
}
