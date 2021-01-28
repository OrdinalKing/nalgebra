use num::Zero;

use simba::scalar::{RealField, SubsetOf, SupersetOf};
use simba::simd::{PrimitiveSimdValue, SimdValue};

#[cfg(feature = "mint")]
use mint;

use crate::base::allocator::Allocator;
use crate::base::dimension::{DimMin, DimName, DimNameAdd, DimNameSum, U1};
use crate::base::{DefaultAllocator, Matrix2, Matrix3, Matrix4, MatrixN, Scalar};

use crate::geometry::{
    AbstractRotation, Isometry, Rotation, Rotation2, Rotation3, Similarity, SuperTCategoryOf,
    TAffine, Transform, Translation, UnitComplex, UnitQuaternion, UnitDualQuaternion,
};

/*
 * This file provides the following conversions:
 * =============================================
 *
 * Rotation  -> Rotation
 * Rotation3 -> UnitQuaternion
 * Rotation3 -> UnitDualQuaternion
 * Rotation2 -> UnitComplex
 * Rotation  -> Isometry
 * Rotation  -> Similarity
 * Rotation  -> Transform
 * Rotation  -> Matrix (homogeneous)
 * mint::EulerAngles -> Rotation

*/

impl<N1, N2, D: DimName> SubsetOf<Rotation<N2, D>> for Rotation<N1, D>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
    DefaultAllocator: Allocator<N1, D, D> + Allocator<N2, D, D>,
{
    #[inline]
    fn to_superset(&self) -> Rotation<N2, D> {
        Rotation::from_matrix_unchecked(self.matrix().to_superset())
    }

    #[inline]
    fn is_in_subset(rot: &Rotation<N2, D>) -> bool {
        crate::is_convertible::<_, MatrixN<N1, D>>(rot.matrix())
    }

    #[inline]
    fn from_superset_unchecked(rot: &Rotation<N2, D>) -> Self {
        Rotation::from_matrix_unchecked(rot.matrix().to_subset_unchecked())
    }
}

impl<N1, N2> SubsetOf<UnitQuaternion<N2>> for Rotation3<N1>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
{
    #[inline]
    fn to_superset(&self) -> UnitQuaternion<N2> {
        let q = UnitQuaternion::<N1>::from_rotation_matrix(self);
        q.to_superset()
    }

    #[inline]
    fn is_in_subset(q: &UnitQuaternion<N2>) -> bool {
        crate::is_convertible::<_, UnitQuaternion<N1>>(q)
    }

    #[inline]
    fn from_superset_unchecked(q: &UnitQuaternion<N2>) -> Self {
        let q: UnitQuaternion<N1> = crate::convert_ref_unchecked(q);
        q.to_rotation_matrix()
    }
}

impl<N1, N2> SubsetOf<UnitDualQuaternion<N2>> for Rotation3<N1>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
{
    #[inline]
    fn to_superset(&self) -> UnitDualQuaternion<N2> {
        let q = UnitQuaternion::<N1>::from_rotation_matrix(self);
        let dq = UnitDualQuaternion::from_rotation(q);
        dq.to_superset()
    }

    #[inline]
    fn is_in_subset(dq: &UnitDualQuaternion<N2>) -> bool {
        crate::is_convertible::<_, UnitQuaternion<N1>>(&dq.rotation()) &&
            dq.translation().vector.is_zero()
    }

    #[inline]
    fn from_superset_unchecked(dq: &UnitDualQuaternion<N2>) -> Self {
        let dq: UnitDualQuaternion<N1> = crate::convert_ref_unchecked(dq);
        dq.rotation().to_rotation_matrix()
    }
}

impl<N1, N2> SubsetOf<UnitComplex<N2>> for Rotation2<N1>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
{
    #[inline]
    fn to_superset(&self) -> UnitComplex<N2> {
        let q = UnitComplex::<N1>::from_rotation_matrix(self);
        q.to_superset()
    }

    #[inline]
    fn is_in_subset(q: &UnitComplex<N2>) -> bool {
        crate::is_convertible::<_, UnitComplex<N1>>(q)
    }

    #[inline]
    fn from_superset_unchecked(q: &UnitComplex<N2>) -> Self {
        let q: UnitComplex<N1> = crate::convert_ref_unchecked(q);
        q.to_rotation_matrix()
    }
}

impl<N1, N2, D: DimName, R> SubsetOf<Isometry<N2, D, R>> for Rotation<N1, D>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
    R: AbstractRotation<N2, D> + SupersetOf<Self>,
    DefaultAllocator: Allocator<N1, D, D> + Allocator<N2, D>,
{
    #[inline]
    fn to_superset(&self) -> Isometry<N2, D, R> {
        Isometry::from_parts(Translation::identity(), crate::convert_ref(self))
    }

    #[inline]
    fn is_in_subset(iso: &Isometry<N2, D, R>) -> bool {
        iso.translation.vector.is_zero()
    }

    #[inline]
    fn from_superset_unchecked(iso: &Isometry<N2, D, R>) -> Self {
        crate::convert_ref_unchecked(&iso.rotation)
    }
}

impl<N1, N2, D: DimName, R> SubsetOf<Similarity<N2, D, R>> for Rotation<N1, D>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
    R: AbstractRotation<N2, D> + SupersetOf<Self>,
    DefaultAllocator: Allocator<N1, D, D> + Allocator<N2, D>,
{
    #[inline]
    fn to_superset(&self) -> Similarity<N2, D, R> {
        Similarity::from_parts(Translation::identity(), crate::convert_ref(self), N2::one())
    }

    #[inline]
    fn is_in_subset(sim: &Similarity<N2, D, R>) -> bool {
        sim.isometry.translation.vector.is_zero() && sim.scaling() == N2::one()
    }

    #[inline]
    fn from_superset_unchecked(sim: &Similarity<N2, D, R>) -> Self {
        crate::convert_ref_unchecked(&sim.isometry.rotation)
    }
}

impl<N1, N2, D, C> SubsetOf<Transform<N2, D, C>> for Rotation<N1, D>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
    C: SuperTCategoryOf<TAffine>,
    D: DimNameAdd<U1> + DimMin<D, Output = D>, // needed by .is_special_orthogonal()
    DefaultAllocator: Allocator<N1, D, D>
        + Allocator<N2, D, D>
        + Allocator<N1, DimNameSum<D, U1>, DimNameSum<D, U1>>
        + Allocator<N2, DimNameSum<D, U1>, DimNameSum<D, U1>>
        + Allocator<(usize, usize), D>,
{
    // needed by .is_special_orthogonal()
    #[inline]
    fn to_superset(&self) -> Transform<N2, D, C> {
        Transform::from_matrix_unchecked(self.to_homogeneous().to_superset())
    }

    #[inline]
    fn is_in_subset(t: &Transform<N2, D, C>) -> bool {
        <Self as SubsetOf<_>>::is_in_subset(t.matrix())
    }

    #[inline]
    fn from_superset_unchecked(t: &Transform<N2, D, C>) -> Self {
        Self::from_superset_unchecked(t.matrix())
    }
}

impl<N1, N2, D> SubsetOf<MatrixN<N2, DimNameSum<D, U1>>> for Rotation<N1, D>
where
    N1: RealField,
    N2: RealField + SupersetOf<N1>,
    D: DimNameAdd<U1> + DimMin<D, Output = D>, // needed by .is_special_orthogonal()
    DefaultAllocator: Allocator<N1, D, D>
        + Allocator<N2, D, D>
        + Allocator<N1, DimNameSum<D, U1>, DimNameSum<D, U1>>
        + Allocator<N2, DimNameSum<D, U1>, DimNameSum<D, U1>>
        + Allocator<(usize, usize), D>,
{
    // needed by .is_special_orthogonal()
    #[inline]
    fn to_superset(&self) -> MatrixN<N2, DimNameSum<D, U1>> {
        self.to_homogeneous().to_superset()
    }

    #[inline]
    fn is_in_subset(m: &MatrixN<N2, DimNameSum<D, U1>>) -> bool {
        let rot = m.fixed_slice::<D, D>(0, 0);
        let bottom = m.fixed_slice::<U1, D>(D::dim(), 0);

        // Scalar types agree.
        m.iter().all(|e| SupersetOf::<N1>::is_in_subset(e)) &&
        // The block part is a rotation.
        rot.is_special_orthogonal(N2::default_epsilon() * crate::convert(100.0)) &&
        // The bottom row is (0, 0, ..., 1)
        bottom.iter().all(|e| e.is_zero()) && m[(D::dim(), D::dim())] == N2::one()
    }

    #[inline]
    fn from_superset_unchecked(m: &MatrixN<N2, DimNameSum<D, U1>>) -> Self {
        let r = m.fixed_slice::<D, D>(0, 0);
        Self::from_matrix_unchecked(crate::convert_unchecked(r.into_owned()))
    }
}

#[cfg(feature = "mint")]
impl<N: RealField> From<mint::EulerAngles<N, mint::IntraXYZ>> for Rotation3<N> {
    fn from(euler: mint::EulerAngles<N, mint::IntraXYZ>) -> Self {
        Self::from_euler_angles(euler.a, euler.b, euler.c)
    }
}

impl<N: RealField> From<Rotation2<N>> for Matrix3<N> {
    #[inline]
    fn from(q: Rotation2<N>) -> Self {
        q.to_homogeneous()
    }
}

impl<N: RealField> From<Rotation2<N>> for Matrix2<N> {
    #[inline]
    fn from(q: Rotation2<N>) -> Self {
        q.into_inner()
    }
}

impl<N: RealField> From<Rotation3<N>> for Matrix4<N> {
    #[inline]
    fn from(q: Rotation3<N>) -> Self {
        q.to_homogeneous()
    }
}

impl<N: RealField> From<Rotation3<N>> for Matrix3<N> {
    #[inline]
    fn from(q: Rotation3<N>) -> Self {
        q.into_inner()
    }
}

impl<N: Scalar + PrimitiveSimdValue, D: DimName> From<[Rotation<N::Element, D>; 2]>
    for Rotation<N, D>
where
    N: From<[<N as SimdValue>::Element; 2]>,
    N::Element: Scalar + Copy,
    DefaultAllocator: Allocator<N, D, D> + Allocator<N::Element, D, D>,
{
    #[inline]
    fn from(arr: [Rotation<N::Element, D>; 2]) -> Self {
        Self::from_matrix_unchecked(MatrixN::from([
            arr[0].clone().into_inner(),
            arr[1].clone().into_inner(),
        ]))
    }
}

impl<N: Scalar + PrimitiveSimdValue, D: DimName> From<[Rotation<N::Element, D>; 4]>
    for Rotation<N, D>
where
    N: From<[<N as SimdValue>::Element; 4]>,
    N::Element: Scalar + Copy,
    DefaultAllocator: Allocator<N, D, D> + Allocator<N::Element, D, D>,
{
    #[inline]
    fn from(arr: [Rotation<N::Element, D>; 4]) -> Self {
        Self::from_matrix_unchecked(MatrixN::from([
            arr[0].clone().into_inner(),
            arr[1].clone().into_inner(),
            arr[2].clone().into_inner(),
            arr[3].clone().into_inner(),
        ]))
    }
}

impl<N: Scalar + PrimitiveSimdValue, D: DimName> From<[Rotation<N::Element, D>; 8]>
    for Rotation<N, D>
where
    N: From<[<N as SimdValue>::Element; 8]>,
    N::Element: Scalar + Copy,
    DefaultAllocator: Allocator<N, D, D> + Allocator<N::Element, D, D>,
{
    #[inline]
    fn from(arr: [Rotation<N::Element, D>; 8]) -> Self {
        Self::from_matrix_unchecked(MatrixN::from([
            arr[0].clone().into_inner(),
            arr[1].clone().into_inner(),
            arr[2].clone().into_inner(),
            arr[3].clone().into_inner(),
            arr[4].clone().into_inner(),
            arr[5].clone().into_inner(),
            arr[6].clone().into_inner(),
            arr[7].clone().into_inner(),
        ]))
    }
}

impl<N: Scalar + PrimitiveSimdValue, D: DimName> From<[Rotation<N::Element, D>; 16]>
    for Rotation<N, D>
where
    N: From<[<N as SimdValue>::Element; 16]>,
    N::Element: Scalar + Copy,
    DefaultAllocator: Allocator<N, D, D> + Allocator<N::Element, D, D>,
{
    #[inline]
    fn from(arr: [Rotation<N::Element, D>; 16]) -> Self {
        Self::from_matrix_unchecked(MatrixN::from([
            arr[0].clone().into_inner(),
            arr[1].clone().into_inner(),
            arr[2].clone().into_inner(),
            arr[3].clone().into_inner(),
            arr[4].clone().into_inner(),
            arr[5].clone().into_inner(),
            arr[6].clone().into_inner(),
            arr[7].clone().into_inner(),
            arr[8].clone().into_inner(),
            arr[9].clone().into_inner(),
            arr[10].clone().into_inner(),
            arr[11].clone().into_inner(),
            arr[12].clone().into_inner(),
            arr[13].clone().into_inner(),
            arr[14].clone().into_inner(),
            arr[15].clone().into_inner(),
        ]))
    }
}
