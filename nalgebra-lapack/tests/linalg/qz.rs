use na::{DMatrix, EuclideanNorm, Norm};
use nl::QZ;
use num_complex::Complex;
use simba::scalar::ComplexField;
use std::cmp;

use crate::proptest::*;
use proptest::{prop_assert, proptest};

proptest! {
    #[test]
    fn qz(n in PROPTEST_MATRIX_DIM) {
        let n = cmp::max(1, cmp::min(n, 10));
        let a = DMatrix::<f64>::new_random(n, n);
        let b = DMatrix::<f64>::new_random(n, n);

        let qz =  QZ::new(a.clone(), b.clone());
        let (vsl,s,t,vsr) = qz.clone().unpack();
        let eigenvalues = qz.raw_eigenvalues();

        prop_assert!(relative_eq!(&vsl * s * vsr.transpose(), a.clone(), epsilon = 1.0e-7));
        prop_assert!(relative_eq!(vsl * t * vsr.transpose(), b.clone(), epsilon = 1.0e-7));

        let a_condition_no = a.clone().try_inverse().and_then(|x| Some(EuclideanNorm.norm(&x)* EuclideanNorm.norm(&a)));
        let b_condition_no = b.clone().try_inverse().and_then(|x| Some(EuclideanNorm.norm(&x)* EuclideanNorm.norm(&b)));

        if a_condition_no.unwrap_or(200000.0) < 5.0 && b_condition_no.unwrap_or(200000.0) < 5.0 {
            let a_c = a.clone().map(|x| Complex::new(x, 0.0));
            let b_c = b.clone().map(|x| Complex::new(x, 0.0));


            for (alpha,beta) in eigenvalues.iter() {
                let l_a = a_c.clone() * Complex::new(*beta, 0.0);
                let l_b = b_c.clone() * *alpha;

                prop_assert!(
                    relative_eq!(
                        (&l_a - &l_b).determinant().modulus(),
                         0.0,
                        epsilon = 1.0e-7));

            };
        };
    }

    #[test]
    fn qz_static(a in matrix4(), b in matrix4()) {
        let qz = QZ::new(a.clone(), b.clone());
        let (vsl,s,t,vsr) = qz.unpack();
        let eigenvalues = qz.raw_eigenvalues();

        prop_assert!(relative_eq!(&vsl * s * vsr.transpose(), a, epsilon = 1.0e-7));
        prop_assert!(relative_eq!(vsl * t * vsr.transpose(), b, epsilon = 1.0e-7));

        let a_condition_no = a.clone().try_inverse().and_then(|x| Some(EuclideanNorm.norm(&x)* EuclideanNorm.norm(&a)));
        let b_condition_no = b.clone().try_inverse().and_then(|x| Some(EuclideanNorm.norm(&x)* EuclideanNorm.norm(&b)));

        if a_condition_no.unwrap_or(200000.0) < 5.0 && b_condition_no.unwrap_or(200000.0) < 5.0 {
            let a_c =a.clone().map(|x| Complex::new(x, 0.0));
            let b_c = b.clone().map(|x| Complex::new(x, 0.0));

            for (alpha,beta) in eigenvalues.iter() {
                let l_a = a_c.clone() * Complex::new(*beta, 0.0);
                let l_b = b_c.clone() * *alpha;

                prop_assert!(
                    relative_eq!(
                        (&l_a - &l_b).determinant().modulus(),
                        0.0,
                        epsilon = 1.0e-7));
            }
        };
    }
}
