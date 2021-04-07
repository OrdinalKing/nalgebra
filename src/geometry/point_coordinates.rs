use std::ops::{Deref, DerefMut};

use crate::base::coordinates::{X, XY, XYZ, XYZW, XYZWA, XYZWAB};
use crate::base::Scalar;

use crate::geometry::Point;

/*
 *
 * Give coordinates to Point{1 .. 6}
 *
 */

macro_rules! deref_impl(
    ($D: expr, $Target: ident $(, $comps: ident)*) => {
        impl<N: Scalar> Deref for Point<N, $D>
            // where DefaultAllocator: Allocator<N, $D>
        {
            type Target = $Target<N>;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &*self.coords
            }
        }

        impl<N: Scalar> DerefMut for Point<N, $D>
            // where DefaultAllocator: Allocator<N, $D>
        {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut *self.coords
            }
        }
    }
);

deref_impl!(1, X, x);
deref_impl!(2, XY, x, y);
deref_impl!(3, XYZ, x, y, z);
deref_impl!(4, XYZW, x, y, z, w);
deref_impl!(5, XYZWA, x, y, z, w, a);
deref_impl!(6, XYZWAB, x, y, z, w, a, b);
