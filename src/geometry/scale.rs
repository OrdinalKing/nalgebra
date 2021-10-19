use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use num::{One, Zero};
use std::fmt;
use std::hash;
#[cfg(feature = "abomonation-serialize")]
use std::io::{Result as IOResult, Write};

#[cfg(feature = "serde-serialize-no-std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "abomonation-serialize")]
use abomonation::Abomonation;

use crate::base::allocator::Allocator;
use crate::base::dimension::{DimNameAdd, DimNameSum, U1};
use crate::base::storage::Owned;
use crate::base::{Const, DefaultAllocator, OMatrix, SVector, Scalar};
use crate::ClosedDiv;
use crate::ClosedMul;

use crate::geometry::Point;

/// A scale which supports non-uniform scaling.
#[repr(C)]
pub struct Scale<T, const D: usize> {
    /// The scale coordinates, i.e., how much is multiplied to a point's coordinates when it is
    /// scaled.
    pub vector: SVector<T, D>,
}

impl<T: fmt::Debug, const D: usize> fmt::Debug for Scale<T, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.vector.as_slice().fmt(formatter)
    }
}

impl<T: Scalar + hash::Hash, const D: usize> hash::Hash for Scale<T, D>
where
    Owned<T, Const<D>>: hash::Hash,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.vector.hash(state)
    }
}

impl<T: Scalar + Copy, const D: usize> Copy for Scale<T, D> {}

impl<T: Scalar, const D: usize> Clone for Scale<T, D>
where
    Owned<T, Const<D>>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Scale::from(self.vector.clone())
    }
}

#[cfg(feature = "bytemuck")]
unsafe impl<T, const D: usize> bytemuck::Zeroable for Scale<T, D>
where
    T: Scalar + bytemuck::Zeroable,
    SVector<T, D>: bytemuck::Zeroable,
{
}

#[cfg(feature = "bytemuck")]
unsafe impl<T, const D: usize> bytemuck::Pod for Scale<T, D>
where
    T: Scalar + bytemuck::Pod,
    SVector<T, D>: bytemuck::Pod,
{
}

#[cfg(feature = "abomonation-serialize")]
impl<T, const D: usize> Abomonation for Scale<T, D>
where
    T: Scalar,
    SVector<T, D>: Abomonation,
{
    unsafe fn entomb<W: Write>(&self, writer: &mut W) -> IOResult<()> {
        self.vector.entomb(writer)
    }

    fn extent(&self) -> usize {
        self.vector.extent()
    }

    unsafe fn exhume<'a, 'b>(&'a mut self, bytes: &'b mut [u8]) -> Option<&'b mut [u8]> {
        self.vector.exhume(bytes)
    }
}

#[cfg(feature = "serde-serialize-no-std")]
impl<T: Scalar, const D: usize> Serialize for Scale<T, D>
where
    Owned<T, Const<D>>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.vector.serialize(serializer)
    }
}

#[cfg(feature = "serde-serialize-no-std")]
impl<'a, T: Scalar, const D: usize> Deserialize<'a> for Scale<T, D>
where
    Owned<T, Const<D>>: Deserialize<'a>,
{
    fn deserialize<Des>(deserializer: Des) -> Result<Self, Des::Error>
    where
        Des: Deserializer<'a>,
    {
        let matrix = SVector::<T, D>::deserialize(deserializer)?;

        Ok(Scale::from(matrix))
    }
}

#[cfg(feature = "rkyv-serialize-no-std")]
mod rkyv_impl {
    use super::Scale;
    use crate::base::SVector;
    use rkyv::{offset_of, project_struct, Archive, Deserialize, Fallible, Serialize};

    impl<T: Archive, const D: usize> Archive for Scale<T, D> {
        type Archived = Scale<T::Archived, D>;
        type Resolver = <SVector<T, D> as Archive>::Resolver;

        fn resolve(
            &self,
            pos: usize,
            resolver: Self::Resolver,
            out: &mut core::mem::MaybeUninit<Self::Archived>,
        ) {
            self.vector.resolve(
                pos + offset_of!(Self::Archived, vector),
                resolver,
                project_struct!(out: Self::Archived => vector),
            );
        }
    }

    impl<T: Serialize<S>, S: Fallible + ?Sized, const D: usize> Serialize<S> for Scale<T, D> {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            self.vector.serialize(serializer)
        }
    }

    impl<T: Archive, _D: Fallible + ?Sized, const D: usize> Deserialize<Scale<T, D>, _D>
        for Scale<T::Archived, D>
    where
        T::Archived: Deserialize<T, _D>,
    {
        fn deserialize(&self, deserializer: &mut _D) -> Result<Scale<T, D>, _D::Error> {
            Ok(Scale {
                vector: self.vector.deserialize(deserializer)?,
            })
        }
    }
}

impl<T: Scalar, const D: usize> Scale<T, D> {
    /// Inverts `self`.
    ///
    /// # Example
    /// ```
    /// # use nalgebra::{Scale2, Scale3};
    /// let t = Scale3::new(1.0, 2.0, 3.0);
    /// assert_eq!(t * t.inverse(), Scale3::identity());
    /// assert_eq!(t.inverse() * t, Scale3::identity());
    ///
    /// // Work in all dimensions.
    /// let t = Scale2::new(1.0, 2.0);
    /// assert_eq!(t * t.inverse(), Scale2::identity());
    /// assert_eq!(t.inverse() * t, Scale2::identity());
    /// ```
    #[inline]
    #[must_use = "Did you mean to use inverse_mut()?"]
    pub fn inverse(&self) -> Scale<T, D>
    where
        T: ClosedDiv + One,
    {
        let useless: SVector<T, D> = SVector::from_element(T::one());
        return useless.component_div(&self.vector).into();
    }

    /// Converts this Scale into its equivalent homogeneous transformation matrix.
    ///
    /// # Example
    /// ```
    /// # use nalgebra::{Scale2, Scale3, Matrix3, Matrix4};
    /// let t = Scale3::new(10.0, 20.0, 30.0);
    /// let expected = Matrix4::new(1.0, 0.0, 0.0, 10.0,
    ///                             0.0, 1.0, 0.0, 20.0,
    ///                             0.0, 0.0, 1.0, 30.0,
    ///                             0.0, 0.0, 0.0, 1.0);
    /// assert_eq!(t.to_homogeneous(), expected);
    ///
    /// let t = Scale2::new(10.0, 20.0);
    /// let expected = Matrix3::new(1.0, 0.0, 10.0,
    ///                             0.0, 1.0, 20.0,
    ///                             0.0, 0.0, 1.0);
    /// assert_eq!(t.to_homogeneous(), expected);
    /// ```
    #[inline]
    #[must_use]
    pub fn to_homogeneous(&self) -> OMatrix<T, DimNameSum<Const<D>, U1>, DimNameSum<Const<D>, U1>>
    where
        T: Zero + One,
        Const<D>: DimNameAdd<U1>,
        DefaultAllocator: Allocator<T, DimNameSum<Const<D>, U1>, DimNameSum<Const<D>, U1>>,
    {
        todo!();
    }

    /// Inverts `self` in-place.
    ///
    /// # Example
    /// ```
    /// # use nalgebra::{Scale2, Scale3};
    /// let t = Scale3::new(1.0, 2.0, 3.0);
    /// let mut inv_t = Scale3::new(1.0, 2.0, 3.0);
    /// inv_t.inverse_mut();
    /// assert_eq!(t * inv_t, Scale3::identity());
    /// assert_eq!(inv_t * t, Scale3::identity());
    ///
    /// // Work in all dimensions.
    /// let t = Scale2::new(1.0, 2.0);
    /// let mut inv_t = Scale2::new(1.0, 2.0);
    /// inv_t.inverse_mut();
    /// assert_eq!(t * inv_t, Scale2::identity());
    /// assert_eq!(inv_t * t, Scale2::identity());
    /// ```
    #[inline]
    pub fn inverse_mut(&mut self)
    where
        T: ClosedDiv + One,
    {
        self.vector = self.inverse().vector;
    }
}

impl<T: Scalar + ClosedMul, const D: usize> Scale<T, D> {
    /// Translate the given point.
    ///
    /// This is the same as the multiplication `self * pt`.
    ///
    /// # Example
    /// ```
    /// # use nalgebra::{Scale3, Point3};
    /// let t = Scale3::new(1.0, 2.0, 3.0);
    /// let transformed_point = t.transform_point(&Point3::new(4.0, 5.0, 6.0));
    /// assert_eq!(transformed_point, Point3::new(5.0, 7.0, 9.0));
    #[inline]
    #[must_use]
    pub fn transform_point(&self, pt: &Point<T, D>) -> Point<T, D> {
        return self * pt;
    }
}

impl<T: Scalar + ClosedDiv + ClosedMul + One, const D: usize> Scale<T, D> {
    /// Translate the given point by the inverse of this Scale.
    ///
    /// # Example
    /// ```
    /// # use nalgebra::{Scale3, Point3};
    /// let t = Scale3::new(1.0, 2.0, 3.0);
    /// let transformed_point = t.inverse_transform_point(&Point3::new(4.0, 5.0, 6.0));
    /// assert_eq!(transformed_point, Point3::new(3.0, 3.0, 3.0));
    #[inline]
    #[must_use]
    pub fn inverse_transform_point(&self, pt: &Point<T, D>) -> Point<T, D> {
        return self.inverse() * pt;
    }
}

impl<T: Scalar + Eq, const D: usize> Eq for Scale<T, D> {}

impl<T: Scalar + PartialEq, const D: usize> PartialEq for Scale<T, D> {
    #[inline]
    fn eq(&self, right: &Scale<T, D>) -> bool {
        self.vector == right.vector
    }
}

impl<T: Scalar + AbsDiffEq, const D: usize> AbsDiffEq for Scale<T, D>
where
    T::Epsilon: Clone,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.vector.abs_diff_eq(&other.vector, epsilon)
    }
}

impl<T: Scalar + RelativeEq, const D: usize> RelativeEq for Scale<T, D>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.vector
            .relative_eq(&other.vector, epsilon, max_relative)
    }
}

impl<T: Scalar + UlpsEq, const D: usize> UlpsEq for Scale<T, D>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.vector.ulps_eq(&other.vector, epsilon, max_ulps)
    }
}

/*
 *
 * Display
 *
 */
impl<T: Scalar + fmt::Display, const D: usize> fmt::Display for Scale<T, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);

        writeln!(f, "Scale {{")?;
        write!(f, "{:.*}", precision, self.vector)?;
        writeln!(f, "}}")
    }
}
