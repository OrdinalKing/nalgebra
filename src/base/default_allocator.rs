//! The default matrix data storage allocator.
//!
//! This will use stack-allocated buffers for matrices with dimensions known at compile-time, and
//! heap-allocated buffers for matrices with at least one dimension unknown at compile-time.

use std::cmp;
use std::mem;
use std::ptr;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

use super::Const;
use crate::base::allocator::{Allocator, Reallocator};
use crate::base::array_storage::ArrayStorage;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::base::dimension::Dynamic;
use crate::base::dimension::{Dim, DimName};
use crate::base::storage::{RawStorage, RawStorageMut};
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::base::vec_storage::VecStorage;
use crate::base::Scalar;
use std::mem::{ManuallyDrop, MaybeUninit};

/*
 *
 * Allocator.
 *
 */
/// An allocator based on `GenericArray` and `VecStorage` for statically-sized and dynamically-sized
/// matrices respectively.
#[derive(Copy, Clone, Debug)]
pub struct DefaultAllocator;

// Static - Static
impl<T: Scalar, const R: usize, const C: usize> Allocator<T, Const<R>, Const<C>>
    for DefaultAllocator
{
    type Buffer = ArrayStorage<T, R, C>;
    type BufferUninit = ArrayStorage<MaybeUninit<T>, R, C>;

    #[inline]
    unsafe fn allocate_uninitialized(_: Const<R>, _: Const<C>) -> MaybeUninit<Self::Buffer> {
        mem::MaybeUninit::<Self::Buffer>::uninit()
    }

    #[inline]
    fn allocate_uninit(_: Const<R>, _: Const<C>) -> ArrayStorage<MaybeUninit<T>, R, C> {
        // SAFETY: An uninitialized `[MaybeUninit<_>; _]` is valid.
        let array: [[MaybeUninit<T>; R]; C] = unsafe { MaybeUninit::uninit().assume_init() };
        ArrayStorage(array)
    }

    #[inline]
    unsafe fn assume_init(uninit: ArrayStorage<MaybeUninit<T>, R, C>) -> ArrayStorage<T, R, C> {
        // Safety:
        // * The caller guarantees that all elements of the array are initialized
        // * `MaybeUninit<T>` and T are guaranteed to have the same layout
        // * `MaybeUnint` does not drop, so there are no double-frees
        // And thus the conversion is safe
        ArrayStorage((&uninit as *const _ as *const [_; C]).read())
    }

    #[inline]
    fn allocate_from_iterator<I: IntoIterator<Item = T>>(
        nrows: Const<R>,
        ncols: Const<C>,
        iter: I,
    ) -> Self::Buffer {
        #[cfg(feature = "no_unsound_assume_init")]
        let mut res: Self::Buffer = unimplemented!();
        #[cfg(not(feature = "no_unsound_assume_init"))]
        let mut res = unsafe { Self::allocate_uninitialized(nrows, ncols).assume_init() };
        let mut count = 0;

        // Safety: this is OK because the Buffer is known to be contiguous.
        let res_slice = unsafe { res.as_mut_slice_unchecked() };
        for (res, e) in res_slice.iter_mut().zip(iter.into_iter()) {
            *res = e;
            count += 1;
        }

        assert!(
            count == nrows.value() * ncols.value(),
            "Matrix init. from iterator: iterator not long enough."
        );

        res
    }
}

// Dynamic - Static
// Dynamic - Dynamic
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, C: Dim> Allocator<T, Dynamic, C> for DefaultAllocator {
    type Buffer = VecStorage<T, Dynamic, C>;
    type BufferUninit = VecStorage<MaybeUninit<T>, Dynamic, C>;

    #[inline]
    unsafe fn allocate_uninitialized(nrows: Dynamic, ncols: C) -> MaybeUninit<Self::Buffer> {
        let mut res = Vec::new();
        let length = nrows.value() * ncols.value();
        res.reserve_exact(length);
        res.set_len(length);

        mem::MaybeUninit::new(VecStorage::new(nrows, ncols, res))
    }

    #[inline]
    fn allocate_uninit(nrows: Dynamic, ncols: C) -> VecStorage<MaybeUninit<T>, Dynamic, C> {
        let mut data = Vec::new();
        let length = nrows.value() * ncols.value();
        data.reserve_exact(length);
        data.resize_with(length, MaybeUninit::uninit);

        VecStorage::new(nrows, ncols, data)
    }

    #[inline]
    unsafe fn assume_init(
        uninit: VecStorage<MaybeUninit<T>, Dynamic, C>,
    ) -> VecStorage<T, Dynamic, C> {
        // Avoids a double-drop.
        let (nrows, ncols) = uninit.shape();
        let vec: Vec<_> = uninit.into();
        let mut md = ManuallyDrop::new(vec);

        // Safety:
        // - MaybeUninit<T> has the same alignment and layout as T.
        // - The length and capacity come from a valid vector.
        let new_data = Vec::from_raw_parts(md.as_mut_ptr() as *mut _, md.len(), md.capacity());

        VecStorage::new(nrows, ncols, new_data)
    }

    #[inline]
    fn allocate_from_iterator<I: IntoIterator<Item = T>>(
        nrows: Dynamic,
        ncols: C,
        iter: I,
    ) -> Self::Buffer {
        let it = iter.into_iter();
        let res: Vec<T> = it.collect();
        assert!(res.len() == nrows.value() * ncols.value(),
                "Allocation from iterator error: the iterator did not yield the correct number of elements.");

        VecStorage::new(nrows, ncols, res)
    }
}

// Static - Dynamic
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, R: DimName> Allocator<T, R, Dynamic> for DefaultAllocator {
    type Buffer = VecStorage<T, R, Dynamic>;
    type BufferUninit = VecStorage<MaybeUninit<T>, R, Dynamic>;

    #[inline]
    unsafe fn allocate_uninitialized(nrows: R, ncols: Dynamic) -> MaybeUninit<Self::Buffer> {
        let mut res = Vec::new();
        let length = nrows.value() * ncols.value();
        res.reserve_exact(length);
        res.set_len(length);

        mem::MaybeUninit::new(VecStorage::new(nrows, ncols, res))
    }

    #[inline]
    fn allocate_uninit(nrows: R, ncols: Dynamic) -> VecStorage<MaybeUninit<T>, R, Dynamic> {
        let mut data = Vec::new();
        let length = nrows.value() * ncols.value();
        data.reserve_exact(length);
        data.resize_with(length, MaybeUninit::uninit);

        VecStorage::new(nrows, ncols, data)
    }

    #[inline]
    unsafe fn assume_init(
        uninit: VecStorage<MaybeUninit<T>, R, Dynamic>,
    ) -> VecStorage<T, R, Dynamic> {
        // Avoids a double-drop.
        let (nrows, ncols) = uninit.shape();
        let vec: Vec<_> = uninit.into();
        let mut md = ManuallyDrop::new(vec);

        // Safety:
        // - MaybeUninit<T> has the same alignment and layout as T.
        // - The length and capacity come from a valid vector.
        let new_data = Vec::from_raw_parts(md.as_mut_ptr() as *mut _, md.len(), md.capacity());

        VecStorage::new(nrows, ncols, new_data)
    }

    #[inline]
    fn allocate_from_iterator<I: IntoIterator<Item = T>>(
        nrows: R,
        ncols: Dynamic,
        iter: I,
    ) -> Self::Buffer {
        let it = iter.into_iter();
        let res: Vec<T> = it.collect();
        assert!(res.len() == nrows.value() * ncols.value(),
                "Allocation from iterator error: the iterator did not yield the correct number of elements.");

        VecStorage::new(nrows, ncols, res)
    }
}

/*
 *
 * Reallocator.
 *
 */
// Anything -> Static × Static
impl<T: Scalar, RFrom, CFrom, const RTO: usize, const CTO: usize>
    Reallocator<T, RFrom, CFrom, Const<RTO>, Const<CTO>> for DefaultAllocator
where
    RFrom: Dim,
    CFrom: Dim,
    Self: Allocator<T, RFrom, CFrom>,
{
    #[inline]
    unsafe fn reallocate_copy(
        rto: Const<RTO>,
        cto: Const<CTO>,
        buf: <Self as Allocator<T, RFrom, CFrom>>::Buffer,
    ) -> ArrayStorage<T, RTO, CTO> {
        #[cfg(feature = "no_unsound_assume_init")]
        let mut res: ArrayStorage<T, RTO, CTO> = unimplemented!();
        #[cfg(not(feature = "no_unsound_assume_init"))]
        let mut res =
            <Self as Allocator<T, Const<RTO>, Const<CTO>>>::allocate_uninitialized(rto, cto)
                .assume_init();

        let (rfrom, cfrom) = buf.shape();

        let len_from = rfrom.value() * cfrom.value();
        let len_to = rto.value() * cto.value();
        ptr::copy_nonoverlapping(buf.ptr(), res.ptr_mut(), cmp::min(len_from, len_to));

        res
    }
}

// Static × Static -> Dynamic × Any
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, CTo, const RFROM: usize, const CFROM: usize>
    Reallocator<T, Const<RFROM>, Const<CFROM>, Dynamic, CTo> for DefaultAllocator
where
    CTo: Dim,
{
    #[inline]
    unsafe fn reallocate_copy(
        rto: Dynamic,
        cto: CTo,
        buf: ArrayStorage<T, RFROM, CFROM>,
    ) -> VecStorage<T, Dynamic, CTo> {
        #[cfg(feature = "no_unsound_assume_init")]
        let mut res: VecStorage<T, Dynamic, CTo> = unimplemented!();
        #[cfg(not(feature = "no_unsound_assume_init"))]
        let mut res =
            <Self as Allocator<T, Dynamic, CTo>>::allocate_uninitialized(rto, cto).assume_init();

        let (rfrom, cfrom) = buf.shape();

        let len_from = rfrom.value() * cfrom.value();
        let len_to = rto.value() * cto.value();
        ptr::copy_nonoverlapping(buf.ptr(), res.ptr_mut(), cmp::min(len_from, len_to));

        res
    }
}

// Static × Static -> Static × Dynamic
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, RTo, const RFROM: usize, const CFROM: usize>
    Reallocator<T, Const<RFROM>, Const<CFROM>, RTo, Dynamic> for DefaultAllocator
where
    RTo: DimName,
{
    #[inline]
    unsafe fn reallocate_copy(
        rto: RTo,
        cto: Dynamic,
        buf: ArrayStorage<T, RFROM, CFROM>,
    ) -> VecStorage<T, RTo, Dynamic> {
        #[cfg(feature = "no_unsound_assume_init")]
        let mut res: VecStorage<T, RTo, Dynamic> = unimplemented!();
        #[cfg(not(feature = "no_unsound_assume_init"))]
        let mut res =
            <Self as Allocator<T, RTo, Dynamic>>::allocate_uninitialized(rto, cto).assume_init();

        let (rfrom, cfrom) = buf.shape();

        let len_from = rfrom.value() * cfrom.value();
        let len_to = rto.value() * cto.value();
        ptr::copy_nonoverlapping(buf.ptr(), res.ptr_mut(), cmp::min(len_from, len_to));

        res
    }
}

// All conversion from a dynamic buffer to a dynamic buffer.
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, CFrom: Dim, CTo: Dim> Reallocator<T, Dynamic, CFrom, Dynamic, CTo>
    for DefaultAllocator
{
    #[inline]
    unsafe fn reallocate_copy(
        rto: Dynamic,
        cto: CTo,
        buf: VecStorage<T, Dynamic, CFrom>,
    ) -> VecStorage<T, Dynamic, CTo> {
        let new_buf = buf.resize(rto.value() * cto.value());
        VecStorage::new(rto, cto, new_buf)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, CFrom: Dim, RTo: DimName> Reallocator<T, Dynamic, CFrom, RTo, Dynamic>
    for DefaultAllocator
{
    #[inline]
    unsafe fn reallocate_copy(
        rto: RTo,
        cto: Dynamic,
        buf: VecStorage<T, Dynamic, CFrom>,
    ) -> VecStorage<T, RTo, Dynamic> {
        let new_buf = buf.resize(rto.value() * cto.value());
        VecStorage::new(rto, cto, new_buf)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, RFrom: DimName, CTo: Dim> Reallocator<T, RFrom, Dynamic, Dynamic, CTo>
    for DefaultAllocator
{
    #[inline]
    unsafe fn reallocate_copy(
        rto: Dynamic,
        cto: CTo,
        buf: VecStorage<T, RFrom, Dynamic>,
    ) -> VecStorage<T, Dynamic, CTo> {
        let new_buf = buf.resize(rto.value() * cto.value());
        VecStorage::new(rto, cto, new_buf)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Scalar, RFrom: DimName, RTo: DimName> Reallocator<T, RFrom, Dynamic, RTo, Dynamic>
    for DefaultAllocator
{
    #[inline]
    unsafe fn reallocate_copy(
        rto: RTo,
        cto: Dynamic,
        buf: VecStorage<T, RFrom, Dynamic>,
    ) -> VecStorage<T, RTo, Dynamic> {
        let new_buf = buf.resize(rto.value() * cto.value());
        VecStorage::new(rto, cto, new_buf)
    }
}
