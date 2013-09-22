//! Vector with dimensions unknown at compile-time.

#[doc(hidden)]; // we hide doc to not have to document the $trhs double dispatch trait.

use std::num::{Zero, One, Algebraic};
use std::rand::Rand;
use std::rand;
use std::vec;
use std::vec::{VecIterator, VecMutIterator};
use std::cmp::ApproxEq;
use std::iter::FromIterator;
use traits::geometry::{Dot, Norm, Translation};
use traits::structure::{Iterable, IterableMut};

mod metal;

/// Vector with a dimension unknown at compile-time.
#[deriving(Eq, ToStr, Clone)]
pub struct DVec<N> {
    /// Components of the vector. Contains as much elements as the vector dimension.
    at: ~[N]
}

double_dispatch_binop_decl_trait!(DVec, DVecMulRhs)
double_dispatch_binop_decl_trait!(DVec, DVecDivRhs)
double_dispatch_binop_decl_trait!(DVec, DVecAddRhs)
double_dispatch_binop_decl_trait!(DVec, DVecSubRhs)

mul_redispatch_impl!(DVec, DVecMulRhs)
div_redispatch_impl!(DVec, DVecDivRhs)
add_redispatch_impl!(DVec, DVecAddRhs)
sub_redispatch_impl!(DVec, DVecSubRhs)

impl<N: Zero + Clone> DVec<N> {
    /// Builds a vector filled with zeros.
    /// 
    /// # Arguments
    ///   * `dim` - The dimension of the vector.
    #[inline]
    pub fn new_zeros(dim: uint) -> DVec<N> {
        DVec::from_elem(dim, Zero::zero())
    }

    /// Tests if all components of the vector are zeroes.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.at.iter().all(|e| e.is_zero())
    }
}

impl<N: Clone> DVec<N> {
    /// Indexing without bounds checking.
    pub unsafe fn at_fast(&self, i: uint) -> N {
        vec::raw::get(self.at, i)
    }
}

impl<N: One + Clone> DVec<N> {
    /// Builds a vector filled with ones.
    /// 
    /// # Arguments
    ///   * `dim` - The dimension of the vector.
    #[inline]
    pub fn new_ones(dim: uint) -> DVec<N> {
        DVec::from_elem(dim, One::one())
    }
}

impl<N: Rand> DVec<N> {
    /// Builds a vector filled with random values.
    #[inline]
    pub fn new_random(dim: uint) -> DVec<N> {
        DVec::from_fn(dim, |_| rand::random())
    }
}

impl<N> DVec<N> {
    /// Creates an uninitialized vec.
    #[inline]
    pub unsafe fn new_uninitialized(dim: uint) -> DVec<N> {
        let mut vec = vec::with_capacity(dim);
        vec::raw::set_len(&mut vec, dim);

        DVec {
            at: vec
        }
    }
}

impl<N: Clone> DVec<N> {
    /// Builds a vector filled with a constant.
    #[inline]
    pub fn from_elem(dim: uint, elem: N) -> DVec<N> {
        DVec { at: vec::from_elem(dim, elem) }
    }
}

impl<N> DVec<N> {
    /// Builds a vector filled with the result of a function.
    #[inline(always)]
    pub fn from_fn(dim: uint, f: &fn(uint) -> N) -> DVec<N> {
        DVec { at: vec::from_fn(dim, |i| f(i)) }
    }
}

impl<N> Container for DVec<N> {
    #[inline]
    fn len(&self) -> uint {
        self.at.len()
    }
}

impl<N> Iterable<N> for DVec<N> {
    #[inline]
    fn iter<'l>(&'l self) -> VecIterator<'l, N> {
        self.at.iter()
    }
}

impl<N> IterableMut<N> for DVec<N> {
    #[inline]
    fn mut_iter<'l>(&'l mut self) -> VecMutIterator<'l, N> {
        self.at.mut_iter()
    }
}

impl<N> FromIterator<N> for DVec<N> {
    #[inline]
    fn from_iterator<I: Iterator<N>>(mut param: &mut I) -> DVec<N> {
        let mut res = DVec { at: ~[] };

        for e in param {
            res.at.push(e)
        }

        res
    }
}

impl<N: Clone + Num + Algebraic + ApproxEq<N> + DVecMulRhs<N, DVec<N>>> DVec<N> {
    /// Computes the canonical basis for the given dimension. A canonical basis is a set of
    /// vectors, mutually orthogonal, with all its component equal to 0.0 exept one which is equal
    /// to 1.0.
    pub fn canonical_basis_with_dim(dim: uint) -> ~[DVec<N>] {
        let mut res : ~[DVec<N>] = ~[];

        for i in range(0u, dim) {
            let mut basis_element : DVec<N> = DVec::new_zeros(dim);

            basis_element.at[i] = One::one();

            res.push(basis_element);
        }

        res
    }

    /// Computes a basis of the space orthogonal to the vector. If the input vector is of dimension
    /// `n`, this will return `n - 1` vectors.
    pub fn orthogonal_subspace_basis(&self) -> ~[DVec<N>] {
        // compute the basis of the orthogonal subspace using Gram-Schmidt
        // orthogonalization algorithm
        let     dim              = self.at.len();
        let mut res : ~[DVec<N>] = ~[];

        for i in range(0u, dim) {
            let mut basis_element : DVec<N> = DVec::new_zeros(self.at.len());

            basis_element.at[i] = One::one();

            if res.len() == dim - 1 {
                break;
            }

            let mut elt = basis_element.clone();

            elt = elt - self * basis_element.dot(self);

            for v in res.iter() {
                elt = elt - v * elt.dot(v)
            };

            if !elt.sqnorm().approx_eq(&Zero::zero()) {
                res.push(elt.normalized());
            }
        }

        assert!(res.len() == dim - 1);

        res
    }
}

impl<N: Add<N, N>> DVecAddRhs<N, DVec<N>> for DVec<N> {
    #[inline]
    fn binop(left: &DVec<N>, right: &DVec<N>) -> DVec<N> {
        assert!(left.at.len() == right.at.len());
        DVec {
            at: left.at.iter().zip(right.at.iter()).map(|(a, b)| *a + *b).collect()
        }
    }
}

impl<N: Sub<N, N>> DVecSubRhs<N, DVec<N>> for DVec<N> {
    #[inline]
    fn binop(left: &DVec<N>, right: &DVec<N>) -> DVec<N> {
        assert!(left.at.len() == right.at.len());
        DVec {
            at: left.at.iter().zip(right.at.iter()).map(|(a, b)| *a - *b).collect()
        }
    }
}

impl<N: Neg<N>> Neg<DVec<N>> for DVec<N> {
    #[inline]
    fn neg(&self) -> DVec<N> {
        DVec { at: self.at.iter().map(|a| -a).collect() }
    }
}

impl<N: Num> Dot<N> for DVec<N> {
    #[inline]
    fn dot(&self, other: &DVec<N>) -> N {
        assert!(self.at.len() == other.at.len());

        let mut res: N = Zero::zero();

        for i in range(0u, self.at.len()) {
            res = res + self.at[i] * other.at[i];
        }

        res
    } 

    #[inline]
    fn sub_dot(&self, a: &DVec<N>, b: &DVec<N>) -> N {
        let mut res: N = Zero::zero();

        for i in range(0u, self.at.len()) {
            res = res + (self.at[i] - a.at[i]) * b.at[i];
        }

        res
    } 
}

impl<N: Add<N, N> + Neg<N> + Clone> Translation<DVec<N>> for DVec<N> {
    #[inline]
    fn translation(&self) -> DVec<N> {
        self.clone()
    }

    #[inline]
    fn inv_translation(&self) -> DVec<N> {
        -self
    }

    #[inline]
    fn translate_by(&mut self, t: &DVec<N>) {
        *self = *self + *t;
    }

    #[inline]
    fn translated(&self, t: &DVec<N>) -> DVec<N> {
        self + *t
    }

    #[inline]
    fn set_translation(&mut self, t: DVec<N>) {
        *self = t
    }
}

impl<N: Num + Algebraic + Clone> Norm<N> for DVec<N> {
    #[inline]
    fn sqnorm(&self) -> N {
        self.dot(self)
    }

    #[inline]
    fn norm(&self) -> N {
        self.sqnorm().sqrt()
    }

    #[inline]
    fn normalized(&self) -> DVec<N> {
        let mut res : DVec<N> = self.clone();

        res.normalize();

        res
    }

    #[inline]
    fn normalize(&mut self) -> N {
        let l = self.norm();

        for i in range(0u, self.at.len()) {
            self.at[i] = self.at[i] / l;
        }

        l
    }
}

impl<N: ApproxEq<N>> ApproxEq<N> for DVec<N> {
    #[inline]
    fn approx_epsilon() -> N {
        fail!("Fix me.")
        // let res: N = ApproxEq::<N>::approx_epsilon();

        // res
    }

    #[inline]
    fn approx_eq(&self, other: &DVec<N>) -> bool {
        let mut zip = self.at.iter().zip(other.at.iter());

        do zip.all |(a, b)| {
            a.approx_eq(b)
        }
    }

    #[inline]
    fn approx_eq_eps(&self, other: &DVec<N>, epsilon: &N) -> bool {
        let mut zip = self.at.iter().zip(other.at.iter());

        do zip.all |(a, b)| {
            a.approx_eq_eps(b, epsilon)
        }
    }
}

macro_rules! scalar_mul_impl (
    ($n: ident) => (
        impl DVecMulRhs<$n, DVec<$n>> for $n {
            #[inline]
            fn binop(left: &DVec<$n>, right: &$n) -> DVec<$n> {
                DVec { at: left.at.iter().map(|a| a * *right).collect() }
            }
        }
    )
)

macro_rules! scalar_div_impl (
    ($n: ident) => (
        impl DVecDivRhs<$n, DVec<$n>> for $n {
            #[inline]
            fn binop(left: &DVec<$n>, right: &$n) -> DVec<$n> {
                DVec { at: left.at.iter().map(|a| a / *right).collect() }
            }
        }
    )
)

macro_rules! scalar_add_impl (
    ($n: ident) => (
        impl DVecAddRhs<$n, DVec<$n>> for $n {
            #[inline]
            fn binop(left: &DVec<$n>, right: &$n) -> DVec<$n> {
                DVec { at: left.at.iter().map(|a| a + *right).collect() }
            }
        }
    )
)

macro_rules! scalar_sub_impl (
    ($n: ident) => (
        impl DVecSubRhs<$n, DVec<$n>> for $n {
            #[inline]
            fn binop(left: &DVec<$n>, right: &$n) -> DVec<$n> {
                DVec { at: left.at.iter().map(|a| a - *right).collect() }
            }
        }
    )
)

scalar_mul_impl!(f64)
scalar_mul_impl!(f32)
scalar_mul_impl!(u64)
scalar_mul_impl!(u32)
scalar_mul_impl!(u16)
scalar_mul_impl!(u8)
scalar_mul_impl!(i64)
scalar_mul_impl!(i32)
scalar_mul_impl!(i16)
scalar_mul_impl!(i8)
scalar_mul_impl!(float)
scalar_mul_impl!(uint)
scalar_mul_impl!(int)

scalar_div_impl!(f64)
scalar_div_impl!(f32)
scalar_div_impl!(u64)
scalar_div_impl!(u32)
scalar_div_impl!(u16)
scalar_div_impl!(u8)
scalar_div_impl!(i64)
scalar_div_impl!(i32)
scalar_div_impl!(i16)
scalar_div_impl!(i8)
scalar_div_impl!(float)
scalar_div_impl!(uint)
scalar_div_impl!(int)

scalar_add_impl!(f64)
scalar_add_impl!(f32)
scalar_add_impl!(u64)
scalar_add_impl!(u32)
scalar_add_impl!(u16)
scalar_add_impl!(u8)
scalar_add_impl!(i64)
scalar_add_impl!(i32)
scalar_add_impl!(i16)
scalar_add_impl!(i8)
scalar_add_impl!(float)
scalar_add_impl!(uint)
scalar_add_impl!(int)

scalar_sub_impl!(f64)
scalar_sub_impl!(f32)
scalar_sub_impl!(u64)
scalar_sub_impl!(u32)
scalar_sub_impl!(u16)
scalar_sub_impl!(u8)
scalar_sub_impl!(i64)
scalar_sub_impl!(i32)
scalar_sub_impl!(i16)
scalar_sub_impl!(i8)
scalar_sub_impl!(float)
scalar_sub_impl!(uint)
scalar_sub_impl!(int)
