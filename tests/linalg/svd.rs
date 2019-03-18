#![cfg_attr(rustfmt, rustfmt_skip)]
use na::{DMatrix, Matrix6};

#[cfg(feature = "arbitrary")]
mod quickcheck_tests {
    use na::{
        DMatrix, DVector, Matrix2, Matrix2x5, Matrix3, Matrix3x5, Matrix4, Matrix5x2, Matrix5x3,
    };
    use std::cmp;
    use core::helper::{RandScalar, RandComplex};


    quickcheck! {
    /*
        fn svd(m: DMatrix<f64>) -> bool {
            if m.len() > 0 {
                let svd = m.clone().svd(true, true);
                let recomp_m = svd.clone().recompose().unwrap();
                let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
                let ds = DMatrix::from_diagonal(&s);

                println!("{}{}", &m, &u * &ds * &v_t);

                s.iter().all(|e| *e >= 0.0) &&
                relative_eq!(&u * ds * &v_t, recomp_m, epsilon = 1.0e-5) &&
                relative_eq!(m, recomp_m, epsilon = 1.0e-5)
            }
            else {
                true
            }
        }

        fn svd_static_5_3(m: Matrix5x3<f64>) -> bool {
            let svd = m.svd(true, true);
            let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
            let ds = Matrix3::from_diagonal(&s);

            s.iter().all(|e| *e >= 0.0) &&
            relative_eq!(m, &u * ds * &v_t, epsilon = 1.0e-5) &&
            u.is_orthogonal(1.0e-5) &&
            v_t.is_orthogonal(1.0e-5)
        }

        fn svd_static_5_2(m: Matrix5x2<f64>) -> bool {
            let svd = m.svd(true, true);
            let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
            let ds = Matrix2::from_diagonal(&s);

            s.iter().all(|e| *e >= 0.0) &&
            relative_eq!(m, &u * ds * &v_t, epsilon = 1.0e-5) &&
            u.is_orthogonal(1.0e-5) &&
            v_t.is_orthogonal(1.0e-5)
        }

        fn svd_static_3_5(m: Matrix3x5<f64>) -> bool {
            let svd = m.svd(true, true);
            let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());

            let ds = Matrix3::from_diagonal(&s);

            s.iter().all(|e| *e >= 0.0) &&
            relative_eq!(m, u * ds * v_t, epsilon = 1.0e-5)
        }

        fn svd_static_2_5(m: Matrix2x5<f64>) -> bool {
            let svd = m.svd(true, true);
            let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
            let ds = Matrix2::from_diagonal(&s);

            s.iter().all(|e| *e >= 0.0) &&
            relative_eq!(m, u * ds * v_t, epsilon = 1.0e-5)
        }


        fn svd_static_square(m: Matrix4<f64>) -> bool {
            let svd = m.svd(true, true);
            let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
            let ds = Matrix4::from_diagonal(&s);

            s.iter().all(|e| *e >= 0.0) &&
            relative_eq!(m, u * ds * v_t, epsilon = 1.0e-5) &&
            u.is_orthogonal(1.0e-5) &&
            v_t.is_orthogonal(1.0e-5)
        }
                        */


        fn svd_static_square_2x2(m: Matrix2<RandComplex<f64>>) -> bool {
            let m = m.map(|e| e.0);
            let svd = m.svd(true, true);
            let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
            let ds = Matrix2::from_diagonal(&s);

            println!("u, s, v_t: {}{}{}", u, s, v_t);
            println!("m: {}", m);
            println!("recomp: {}", u * ds * v_t);
            println!("uu_t, vv_t: {}{}", u * u.conjugate_transpose(), v_t.conjugate_transpose() * v_t);

            s.iter().all(|e| e.re >= 0.0) &&
            relative_eq!(m, u * ds * v_t, epsilon = 1.0e-5) &&
            u.is_orthogonal(1.0e-5) &&
            v_t.is_orthogonal(1.0e-5)
        }

/*
        fn svd_pseudo_inverse(m: DMatrix<f64>) -> bool {
            if m.len() > 0 {
                let svd = m.clone().svd(true, true);
                let pinv = svd.pseudo_inverse(1.0e-10).unwrap();

                if m.nrows() > m.ncols() {
                    println!("{}", &pinv * &m);
                    (pinv * m).is_identity(1.0e-5)
                }
                else {
                    println!("{}", &m * &pinv);
                    (m * pinv).is_identity(1.0e-5)
                }
            }
            else {
                true
            }
        }

        fn svd_solve(n: usize, nb: usize) -> bool {
            let n = cmp::max(1, cmp::min(n, 10));
            let nb = cmp::min(nb, 10);
            let m  = DMatrix::<f64>::new_random(n, n);

            let svd = m.clone().svd(true, true);

            if svd.rank(1.0e-7) == n {
                let b1 = DVector::new_random(n);
                let b2 = DMatrix::new_random(n, nb);

                let sol1 = svd.solve(&b1, 1.0e-7).unwrap();
                let sol2 = svd.solve(&b2, 1.0e-7).unwrap();

                let recomp = svd.recompose().unwrap();
                if !relative_eq!(m, recomp, epsilon = 1.0e-6) {
                    println!("{}{}", m, recomp);
                }

                if !relative_eq!(&m * &sol1, b1, epsilon = 1.0e-6) {
                    println!("Problem 1: {:.6}{:.6}", b1, &m * sol1);
                    return false;
                }
                if !relative_eq!(&m * &sol2, b2, epsilon = 1.0e-6) {
                    println!("Problem 2: {:.6}{:.6}", b2, &m * sol2);
                    return false;
                }
            }

            true
        }
        */
    }
}

/*
// Test proposed on the issue #176 of rulinalg.
#[test]
fn svd_singular() {
    let m = DMatrix::from_row_slice(24, 24, &[
        1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  0.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        -1.0, -1.0, -1.0, -1.0, -1.0,  0.0,  1.0,  0.0,  0.0,  1.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0, -1.0,  0.0,  0.0,  0.0,  0.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0,  0.0,  0.0,  0.0,  0.0,  1.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0,  0.0,  1.0,  1.0,  1.0,
        0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0]);

    let svd = m.clone().svd(true, true);
    let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
    let ds = DMatrix::from_diagonal(&s);

    println!("{:.5}", &u * &ds * &v_t);

    assert!(s.iter().all(|e| *e >= 0.0));
    assert!(u.is_orthogonal(1.0e-5));
    assert!(v_t.is_orthogonal(1.0e-5));
    assert!(relative_eq!(m, &u * ds * &v_t, epsilon = 1.0e-5));
}

// Same as the previous test but with one additional row.
#[test]
fn svd_singular_vertical() {
    let m = DMatrix::from_row_slice(25, 24, &[
        1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  0.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        -1.0, -1.0, -1.0, -1.0, -1.0,  0.0,  1.0,  0.0,  0.0,  1.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0, -1.0,  0.0,  0.0,  0.0,  0.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0,  0.0,  0.0,  0.0,  0.0,  1.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0,  0.0,  1.0,  1.0,  1.0,
        0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0]);


    let svd = m.clone().svd(true, true);
    let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
    let ds = DMatrix::from_diagonal(&s);

    assert!(s.iter().all(|e| *e >= 0.0));
    assert!(relative_eq!(m, &u * ds * &v_t, epsilon = 1.0e-5));
}

// Same as the previous test but with one additional column.
#[test]
fn svd_singular_horizontal() {
    let m = DMatrix::from_row_slice(24, 25, &[
        1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  0.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        -1.0, -1.0, -1.0, -1.0, -1.0,  0.0,  1.0,  0.0,  0.0,  1.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0, -1.0,  0.0,  0.0,  0.0,  0.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0,  0.0,  0.0,  0.0,  0.0,  1.0,  1.0,  1.0,  1.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0,  0.0,  1.0,  1.0,  1.0,   1.0,
        0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0,   0.0,
        0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0, -4.0,  0.0,  0.0,  0.0,  0.0,  4.0,  0.0,  0.0,  0.0,  0.0,  0.0, 0.0]);

    let svd = m.clone().svd(true, true);
    let (u, s, v_t) = (svd.u.unwrap(), svd.singular_values, svd.v_t.unwrap());
    let ds = DMatrix::from_diagonal(&s);

    assert!(s.iter().all(|e| *e >= 0.0));
    assert!(relative_eq!(m, &u * ds * &v_t, epsilon = 1.0e-5));
}

#[test]
fn svd_zeros() {
    let m = DMatrix::from_element(10, 10, 0.0);
    let svd = m.clone().svd(true, true);
    assert_eq!(Ok(m), svd.recompose());
}

#[test]
fn svd_identity() {
    let m = DMatrix::<f64>::identity(10, 10);
    let svd = m.clone().svd(true, true);
    assert_eq!(Ok(m), svd.recompose());

    let m = DMatrix::<f64>::identity(10, 15);
    let svd = m.clone().svd(true, true);
    assert_eq!(Ok(m), svd.recompose());

    let m = DMatrix::<f64>::identity(15, 10);
    let svd = m.clone().svd(true, true);
    assert_eq!(Ok(m), svd.recompose());
}

#[test]
fn svd_with_delimited_subproblem() {
    let mut m = DMatrix::<f64>::from_element(10, 10, 0.0);
    m[(0,0)] = 1.0;  m[(0,1)] = 2.0;
    m[(1,1)] = 0.0;  m[(1,2)] = 3.0;
    m[(2,2)] = 4.0;  m[(2,3)] = 5.0;
    m[(3,3)] = 6.0;  m[(3,4)] = 0.0;
    m[(4,4)] = 8.0;  m[(3,5)] = 9.0;
    m[(5,5)] = 10.0; m[(3,6)] = 11.0;
    m[(6,6)] = 12.0; m[(3,7)] = 12.0;
    m[(7,7)] = 14.0; m[(3,8)] = 13.0;
    m[(8,8)] = 16.0; m[(3,9)] = 17.0;
    m[(9,9)] = 18.0;
    let svd = m.clone().svd(true, true);
    assert!(relative_eq!(m, svd.recompose().unwrap(), epsilon = 1.0e-7));

    // Rectangular versions.
    let mut m = DMatrix::<f64>::from_element(15, 10, 0.0);
    m[(0,0)] = 1.0;  m[(0,1)] = 2.0;
    m[(1,1)] = 0.0;  m[(1,2)] = 3.0;
    m[(2,2)] = 4.0;  m[(2,3)] = 5.0;
    m[(3,3)] = 6.0;  m[(3,4)] = 0.0;
    m[(4,4)] = 8.0;  m[(3,5)] = 9.0;
    m[(5,5)] = 10.0; m[(3,6)] = 11.0;
    m[(6,6)] = 12.0; m[(3,7)] = 12.0;
    m[(7,7)] = 14.0; m[(3,8)] = 13.0;
    m[(8,8)] = 16.0; m[(3,9)] = 17.0;
    m[(9,9)] = 18.0;
    let svd = m.clone().svd(true, true);
    assert!(relative_eq!(m, svd.recompose().unwrap(), epsilon = 1.0e-7));

    let svd = m.transpose().svd(true, true);
    assert!(relative_eq!(m.transpose(), svd.recompose().unwrap(), epsilon = 1.0e-7));
}

#[test]
fn svd_fail() {
    let m = Matrix6::new(
        0.9299319121545955,   0.9955870335651049,   0.8824725266413644,  0.28966880207132295,  0.06102723649846409,   0.9311880746048009,
        0.5938395242304351,   0.8398522876024204,  0.06672831951963198,   0.9941213119963099,   0.9431846038057834,   0.8159885168706427,
        0.9121962883152357,   0.6471119669367571,   0.4823309702814407,   0.6420516076705516,   0.7731203925207113,   0.7424069470756647,
        0.07311092531259344,   0.5579247949052946,  0.14518764691585773,  0.03502980663114896,   0.7991329455957719,   0.4929930019965745,
        0.12293810556077789,   0.6617084679545999,   0.9002240700227326, 0.027153062135304884,   0.3630189466989524,  0.18207502727558866,
        0.843196731466686,  0.08951878746549924,   0.7533450877576973, 0.009558876499740077,   0.9429679490873482,   0.9355764454129878);
    let svd = m.clone().svd(true, true);
    println!("Singular values: {}", svd.singular_values);
    println!("u: {:.5}", svd.u.unwrap());
    println!("v: {:.5}", svd.v_t.unwrap());
    let recomp = svd.recompose().unwrap();
    println!("{:.5}{:.5}", m, recomp);
    assert!(relative_eq!(m, recomp, epsilon = 1.0e-5));
}

#[test]
fn svd_err() {
    let m = DMatrix::from_element(10, 10, 0.0);
    let svd = m.clone().svd(false, false);
    assert_eq!(Err("SVD recomposition: U and V^t have not been computed."), svd.clone().recompose());
    assert_eq!(Err("SVD pseudo inverse: the epsilon must be non-negative."), svd.clone().pseudo_inverse(-1.0));
}

*/