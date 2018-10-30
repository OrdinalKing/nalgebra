use alga::general::{ClosedAdd, ClosedMul};
use num::{One, Zero};
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Add, Mul, Range};
use std::slice;

use allocator::Allocator;
use constraint::{AreMultipliable, DimEq, SameNumberOfRows, ShapeConstraint};
use sparse::{CsMatrix, CsStorage, CsStorageIter, CsVecStorage, CsVector};
use storage::{Storage, StorageMut};
use {DefaultAllocator, Dim, Matrix, MatrixMN, Real, Scalar, Vector, VectorN, U1};

pub struct CsCholesky<N: Real, D: Dim>
where
    DefaultAllocator: Allocator<usize, D> + Allocator<N, D>,
{
    // Non-zero pattern of the original matrix upper-triangular part.
    // Unlike the original matrix, the `original_p` array does contain the last sentinel value
    // equal to `original_i.len()` at the end.
    original_p: Vec<usize>,
    original_i: Vec<usize>,
    original_len: usize, // Number of elements on the numerical value vector of the original matrix.
    // Decomposition result.
    l: CsMatrix<N, D, D>,
    // Used only for the pattern.
    // FIXME: store only the nonzero pattern instead.
    u: CsMatrix<N, D, D>,
    ok: bool,
    // Workspaces.
    work_x: VectorN<N, D>,
    work_c: VectorN<usize, D>,
}

impl<N: Real, D: Dim> CsCholesky<N, D>
where
    DefaultAllocator: Allocator<usize, D> + Allocator<N, D>,
{
    /// Computes the cholesky decomposition of the sparse matrix `m`.
    pub fn new(m: &CsMatrix<N, D, D>) -> Self {
        let mut me = Self::new_symbolic(m);
        let _ = me.decompose(&m.data.vals);
        me
    }
    /// Perform symbolic analysis for the given matrix.
    ///
    /// This does not access the numerical values of `m`.
    pub fn new_symbolic(m: &CsMatrix<N, D, D>) -> Self {
        assert!(
            m.is_square(),
            "The matrix `m` must be square to compute its elimination tree."
        );

        let (l, u) = Self::nonzero_pattern(m);

        // Workspaces.
        let work_x = unsafe { VectorN::new_uninitialized_generic(m.data.shape().0, U1) };
        let work_c = unsafe { VectorN::new_uninitialized_generic(m.data.shape().1, U1) };
        let mut original_p = m.data.p.as_slice().to_vec();
        original_p.push(m.data.i.len());

        CsCholesky {
            original_p,
            original_i: m.data.i.clone(),
            original_len: m.data.i.len(),
            l,
            u,
            ok: false,
            work_x,
            work_c,
        }
    }

    pub fn l(&self) -> Option<&CsMatrix<N, D, D>> {
        if self.ok {
            Some(&self.l)
        } else {
            None
        }
    }

    pub fn unwrap_l(self) -> Option<CsMatrix<N, D, D>> {
        if self.ok {
            Some(self.l)
        } else {
            None
        }
    }

    // Performs the numerical Cholesky decomposition given the set of numerical values.
    pub fn decompose(&mut self, values: &[N]) -> bool {
        assert!(
            values.len() >= self.original_len,
            "The set of values is too small."
        );

        // Reset `work_c` to the column pointers of `l`.
        self.work_c.copy_from(&self.l.data.p);

        // Perform the decomposition.
        for k in 0..self.l.nrows() {
            // Scatter the k-th column of the original matrix with the values provided.
            let column_range = self.original_p[k]..self.original_p[k + 1];

            self.work_x[k] = N::zero();
            for p in column_range.clone() {
                let irow = self.original_i[p];

                if irow <= k {
                    self.work_x[irow] = values[p];
                }
            }

            let mut diag = self.work_x[k];
            self.work_x[k] = N::zero();

            // Triangular solve.
            for irow in self.u.data.column_row_indices(k) {
                if irow >= k {
                    continue;
                }

                let lki = self.work_x[irow] / self.l.data.vals[self.l.data.p[irow]];
                self.work_x[irow] = N::zero();

                for p in self.l.data.p[irow] + 1..self.work_c[irow] {
                    self.work_x[self.l.data.i[p]] -= self.l.data.vals[p] * lki;
                }

                diag -= lki * lki;
                let p = self.work_c[irow];
                self.work_c[irow] += 1;
                self.l.data.i[p] = k;
                self.l.data.vals[p] = lki;
            }

            if diag <= N::zero() {
                self.ok = false;
                return false;
            }

            // Deal with the diagonal element.
            let p = self.work_c[k];
            self.work_c[k] += 1;
            self.l.data.i[p] = k;
            self.l.data.vals[p] = diag.sqrt();
        }

        self.ok = true;
        true
    }

    fn elimination_tree<S: CsStorage<N, D, D>>(m: &CsMatrix<N, D, D, S>) -> Vec<usize> {
        let nrows = m.nrows();
        let mut forest: Vec<_> = iter::repeat(usize::max_value()).take(nrows).collect();
        let mut ancestor: Vec<_> = iter::repeat(usize::max_value()).take(nrows).collect();

        for k in 0..nrows {
            for irow in m.data.column_row_indices(k) {
                let mut i = irow;

                while i < k {
                    let i_ancestor = ancestor[i];
                    ancestor[i] = k;

                    if i_ancestor == usize::max_value() {
                        forest[i] = k;
                        break;
                    }

                    i = i_ancestor;
                }
            }
        }

        forest
    }

    fn reach<S: CsStorage<N, D, D>>(
        m: &CsMatrix<N, D, D, S>,
        j: usize,
        max_j: usize,
        tree: &[usize],
        marks: &mut Vec<bool>,
        out: &mut Vec<usize>,
    ) {
        marks.clear();
        marks.resize(tree.len(), false);

        // FIXME: avoid all those allocations.
        let mut tmp = Vec::new();
        let mut res = Vec::new();

        for irow in m.data.column_row_indices(j) {
            let mut curr = irow;
            while curr != usize::max_value() && curr <= max_j && !marks[curr] {
                marks[curr] = true;
                tmp.push(curr);
                curr = tree[curr];
            }

            tmp.append(&mut res);
            mem::swap(&mut tmp, &mut res);
        }

        out.append(&mut res);
    }

    fn nonzero_pattern<S: CsStorage<N, D, D>>(
        m: &CsMatrix<N, D, D, S>,
    ) -> (CsMatrix<N, D, D>, CsMatrix<N, D, D>) {
        let etree = Self::elimination_tree(m);
        let (nrows, ncols) = m.data.shape();
        let mut rows = Vec::with_capacity(m.len());
        let mut cols = unsafe { VectorN::new_uninitialized_generic(m.data.shape().0, U1) };
        let mut marks = Vec::new();

        // NOTE: the following will actually compute the non-zero pattern of
        // the transpose of l.
        for i in 0..nrows.value() {
            cols[i] = rows.len();
            Self::reach(m, i, i, &etree, &mut marks, &mut rows);
        }

        let mut vals = Vec::with_capacity(rows.len());
        unsafe {
            vals.set_len(rows.len());
        }
        vals.shrink_to_fit();

        let data = CsVecStorage {
            shape: (nrows, ncols),
            p: cols,
            i: rows,
            vals,
        };

        let u = CsMatrix::from_data(data);
        // XXX: avoid this transpose.
        let l = u.transpose();

        (l, u)
    }

    /*
     *
     * NOTE: All the following methods are untested and currently unused.
     *
     *
    fn column_counts<S: CsStorage<N, D, D>>(
        m: &CsMatrix<N, D, D, S>,
        tree: &[usize],
    ) -> Vec<usize> {
        let len = m.data.shape().0.value();
        let mut counts: Vec<_> = iter::repeat(0).take(len).collect();
        let mut reach = Vec::new();
        let mut marks = Vec::new();
    
        for i in 0..len {
            Self::reach(m, i, i, tree, &mut marks, &mut reach);
    
            for j in reach.drain(..) {
                counts[j] += 1;
            }
        }
    
        counts
    }
    
    fn tree_postorder(tree: &[usize]) -> Vec<usize> {
        // FIXME: avoid all those allocations?
        let mut first_child: Vec<_> = iter::repeat(usize::max_value()).take(tree.len()).collect();
        let mut other_children: Vec<_> =
            iter::repeat(usize::max_value()).take(tree.len()).collect();
    
        // Build the children list from the parent list.
        // The set of children of the node `i` is given by the linked list
        // starting at `first_child[i]`. The nodes of this list are then:
        // { first_child[i], other_children[first_child[i]], other_children[other_children[first_child[i]], ... }
        for (i, parent) in tree.iter().enumerate() {
            if *parent != usize::max_value() {
                let brother = first_child[*parent];
                first_child[*parent] = i;
                other_children[i] = brother;
            }
        }
    
        let mut stack = Vec::with_capacity(tree.len());
        let mut postorder = Vec::with_capacity(tree.len());
    
        for (i, node) in tree.iter().enumerate() {
            if *node == usize::max_value() {
                Self::dfs(
                    i,
                    &mut first_child,
                    &other_children,
                    &mut stack,
                    &mut postorder,
                )
            }
        }
    
        postorder
    }
    
    fn dfs(
        i: usize,
        first_child: &mut [usize],
        other_children: &[usize],
        stack: &mut Vec<usize>,
        result: &mut Vec<usize>,
    ) {
        stack.clear();
        stack.push(i);
    
        while let Some(n) = stack.pop() {
            let child = first_child[n];
    
            if child == usize::max_value() {
                // No children left.
                result.push(n);
            } else {
                stack.push(n);
                stack.push(child);
                first_child[n] = other_children[child];
            }
        }
    }
    */
}
