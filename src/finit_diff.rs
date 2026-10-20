use ndarray::Array2;
//const PEREODIC:bool  = crate::PEREODIC;

/// Solves `laplacian(psi) = -phi` with successive over-relaxation.
///
/// With `periodic = true`, the first and last x rows are ghost rows; y
/// boundaries remain fixed. Convergence requires the absolute discrete residual
/// `laplace(psi) + phi * h * h` to be below `1e-6` at every interior node.
/// Returns the zero-based index of the converged sweep, or 1000 if not converged.
///
/// # Panics
/// Panics if the array dimensions differ or the grid is smaller than 3x3.
pub fn poisson_relax(phi: &Array2<f64>, psi: &mut Array2<f64>, h: f64, periodic: bool) -> usize {
    assert_eq!(phi.dim(), psi.dim(), "array dimensions must match");
    assert!(
        psi.nrows() >= 3 && psi.ncols() >= 3,
        "grid must be at least 3x3"
    );
    const OMEGA: f64 = 1.8;
    let shape = psi.dim();
    let nx = shape.0;
    let ny = shape.1;
    let h2 = h * h;
    if periodic {
        sync_periodic_x(psi);
    }
    // SAFETY: Both arrays have the same dimensions, each at least three.
    // The interior loop and its immediate neighbors are therefore in bounds.
    unsafe {
        for k in 0..1000 {
            let mut nev = 0.0;
            for i in 1..shape.0 - 1 {
                // Read the actual periodic neighbors, including updates made
                // earlier in this sweep, rather than stale ghost-row values.
                let left = if periodic && i == 1 { nx - 2 } else { i - 1 };
                let right = if periodic && i == nx - 2 { 1 } else { i + 1 };
                for j in 1..shape.1 - 1 {
                    let tmp = -OMEGA * (*psi.uget((i, j)))
                        + OMEGA / 4.0
                            * (*psi.uget((right, j))
                                + *psi.uget((left, j))
                                + *psi.uget((i, j + 1))
                                + *psi.uget((i, j - 1))
                                + *phi.uget((i, j)) * h2);

                    *psi.uget_mut((i, j)) += tmp;
                    if !tmp.is_finite() {
                        nev = f64::INFINITY;
                    } else if nev < tmp.abs() {
                        nev = tmp.abs();
                    }
                }
            }
            if periodic {
                sync_periodic_x(psi);
            }

            if nev < 1e-6 {
                // Only perform the extra residual pass when the cheap update
                // criterion suggests convergence. NaN also fails this check.
                let converged = (1..nx - 1).all(|i| {
                    (1..ny - 1).all(|j| {
                        let residual = *psi.uget((i + 1, j))
                            + *psi.uget((i - 1, j))
                            + *psi.uget((i, j + 1))
                            + *psi.uget((i, j - 1))
                            - 4.0 * *psi.uget((i, j))
                            + *phi.uget((i, j)) * h2;
                        residual.abs() < 1e-6
                    })
                });
                if converged {
                    return k;
                }
            }
        }
        1000
    }
}

fn sync_periodic_x(psi: &mut Array2<f64>) {
    let (nx, ny) = psi.dim();
    for j in 0..ny {
        psi[[0, j]] = psi[[nx - 2, j]];
        psi[[nx - 1, j]] = psi[[1, j]];
    }
}

pub fn laplace(v: &Array2<f64>) -> Array2<f64> {
    let shape = v.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 1..shape.0 - 1 {
        for i in 1..shape.1 - 1 {
            delta[[j, i]] =
                v[[j + 1, i]] + v[[j - 1, i]] + v[[j, i + 1]] + v[[j, i - 1]] - 4.0 * v[[j, i]];
        }
    }
    /*
     let tmp = -4. * &v.slice(s![1..-1, 1..-1])
         + v.slice(s![ ..-2, 1..-1])
         + v.slice(s![1..-1,  ..-2])
         + v.slice(s![1..-1, 2..  ])
         + v.slice(s![2..  , 1..-1]);
    delta.slice_mut(s![1..-1, 1..-1]).assign(&tmp);
    */
    delta
}

pub fn dx(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 1..shape.0 - 1 {
        for i in 1..shape.1 - 1 {
            delta[[j, i]] = (arr[[j + 1, i]] - arr[[j - 1, i]]) / (2.0);
        }
    }
    delta
}
pub fn dx_f(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 0..shape.0 - 1 {
        for i in 0..shape.1 {
            delta[[j, i]] = arr[[j + 1, i]] - arr[[j, i]];
        }
    }
    delta
}
pub fn dy_f(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 0..shape.0 {
        for i in 0..shape.1 - 1 {
            delta[[j, i]] = arr[[j, i + 1]] - arr[[j, i]];
        }
    }
    delta
}

pub fn dx_b(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 1..shape.0 {
        for i in 0..shape.1 {
            delta[[j, i]] = arr[[j, i]] - arr[[j - 1, i]];
        }
    }
    delta
}
pub fn dy_b(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 0..shape.0 {
        for i in 1..shape.1 {
            delta[[j, i]] = arr[[j, i]] - arr[[j, i - 1]];
        }
    }
    delta
}
pub fn mean_cx(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 1..shape.0 {
        for i in 0..shape.1 {
            delta[[j, i]] = (arr[[j, i]] + arr[[j - 1, i]]) / 2.0;
        }
    }
    delta
}
pub fn mean_cy(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 0..shape.0 {
        for i in 1..shape.1 {
            delta[[j, i]] = (arr[[j, i]] + arr[[j, i - 1]]) / 2.0;
        }
    }
    delta
}

pub fn dy(arr: &Array2<f64>) -> Array2<f64> {
    let shape = arr.dim();
    let mut delta = Array2::<f64>::zeros(shape);
    for j in 1..shape.0 - 1 {
        for i in 1..shape.1 - 1 {
            delta[[j, i]] = (arr[[j, i + 1]] - arr[[j, i - 1]]) / (2.0);
        }
    }

    //let arr = array![0, 1, 2, 3];
    //assert_eq!(arr.slice(s![1..3;-1]), array![2, 1]);
    //     let tmp=(&arr.slice(s![1..-1, 2..])-&arr.slice(s![1..-1, 0..-2]))/2.0;

    //   delta.slice_mut(s![1..-1, 1..-1]).assign(&tmp);
    //println!("{:?}",tmp.shape());
    //delta= &x.slice(s![1..]) - &x.slice(s![..-1]);
    delta
}

/// Writes centered y differences at interior nodes, leaving boundaries unchanged.
///
/// # Panics
/// Panics if the array dimensions differ or the grid is smaller than 3x3.
pub fn dy_mut(arr: &Array2<f64>, delta: &mut Array2<f64>) {
    assert_eq!(arr.dim(), delta.dim(), "array dimensions must match");
    assert!(
        arr.nrows() >= 3 && arr.ncols() >= 3,
        "grid must be at least 3x3"
    );
    let shape = arr.dim();
    // SAFETY: The checked dimensions keep interior output indices and both
    // neighboring input indices in bounds.
    unsafe {
        for j in 1..shape.0 - 1 {
            for i in 1..shape.1 - 1 {
                let tmp = (*arr.uget((j, i + 1)) - *arr.uget((j, i - 1))) / (2.0);
                *delta.uget_mut((j, i)) = tmp;
            }
        }
    }
}
/// Writes centered x differences at interior nodes, leaving boundaries unchanged.
///
/// # Panics
/// Panics if the array dimensions differ or the grid is smaller than 3x3.
pub fn dx_mut(arr: &Array2<f64>, delta: &mut Array2<f64>) {
    assert_eq!(arr.dim(), delta.dim(), "array dimensions must match");
    assert!(
        arr.nrows() >= 3 && arr.ncols() >= 3,
        "grid must be at least 3x3"
    );
    let shape = arr.dim();
    // SAFETY: The checked dimensions keep interior output indices and both
    // neighboring input indices in bounds.
    unsafe {
        for j in 1..shape.0 - 1 {
            for i in 1..shape.1 - 1 {
                let tmp = (*arr.uget((j + 1, i)) - *arr.uget((j - 1, i))) / (2.0);
                *delta.uget_mut((j, i)) = tmp;
            }
        }
    }
}
/// Writes the discrete Laplacian at interior nodes, leaving boundaries unchanged.
///
/// # Panics
/// Panics if the array dimensions differ or the grid is smaller than 3x3.
pub fn laplace_mut(v: &Array2<f64>, delta: &mut Array2<f64>) {
    assert_eq!(v.dim(), delta.dim(), "array dimensions must match");
    assert!(
        v.nrows() >= 3 && v.ncols() >= 3,
        "grid must be at least 3x3"
    );
    let shape = v.dim();
    // SAFETY: The checked dimensions keep interior output indices and all four
    // neighboring input indices in bounds.
    unsafe {
        for j in 1..shape.0 - 1 {
            for i in 1..shape.1 - 1 {
                let tmp = *v.uget((j + 1, i))
                    + *v.uget((j - 1, i))
                    + *v.uget((j, i + 1))
                    + *v.uget((j, i - 1))
                    - *v.uget((j, i)) * 4.0;
                *delta.uget_mut((j, i)) = tmp;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{dx_mut, dy_mut, laplace_mut, poisson_relax};
    use ndarray::Array2;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    type Operator = fn(&Array2<f64>, &mut Array2<f64>);
    const OPERATORS: [Operator; 3] = [dx_mut, dy_mut, laplace_mut];

    #[test]
    fn operators_compute_rectangular_grid_and_preserve_boundaries() {
        let input = Array2::from_shape_fn((5, 7), |(i, j)| (i * i + 3 * j * j) as f64);
        for (index, operator) in OPERATORS.into_iter().enumerate() {
            let mut output = Array2::from_elem(input.dim(), -123.0);
            operator(&input, &mut output);
            for ((i, j), &value) in output.indexed_iter() {
                let expected = if i == 0 || i == 4 || j == 0 || j == 6 {
                    -123.0
                } else {
                    match index {
                        0 => 2.0 * i as f64,
                        1 => 6.0 * j as f64,
                        _ => 8.0,
                    }
                };
                assert_eq!(value, expected);
            }
        }
    }

    #[test]
    fn unchecked_routines_reject_mismatched_shapes_before_writing() {
        let input = Array2::zeros((5, 7));
        for shape in [(1, 1), (5, 6), (4, 7), (7, 5)] {
            for operator in OPERATORS {
                let mut output = Array2::from_elem(shape, -123.0);
                assert!(catch_unwind(AssertUnwindSafe(|| operator(&input, &mut output))).is_err());
                assert!(output.iter().all(|&value| value == -123.0));
            }
            let mut psi = Array2::from_elem(shape, -123.0);
            assert!(catch_unwind(AssertUnwindSafe(|| {
                poisson_relax(&input, &mut psi, 1.0, false)
            }))
            .is_err());
            assert!(psi.iter().all(|&value| value == -123.0));
        }
    }

    #[test]
    fn unchecked_routines_reject_small_grids() {
        for shape in [(0, 0), (0, 5), (5, 0), (1, 5), (5, 1), (2, 5), (5, 2)] {
            let input = Array2::zeros(shape);
            for operator in OPERATORS {
                let mut output = Array2::zeros(shape);
                assert!(catch_unwind(AssertUnwindSafe(|| operator(&input, &mut output))).is_err());
            }
            for periodic in [false, true] {
                let mut psi = Array2::zeros(shape);
                assert!(catch_unwind(AssertUnwindSafe(|| {
                    poisson_relax(&input, &mut psi, 1.0, periodic)
                }))
                .is_err());
            }
        }
    }

    #[test]
    fn poisson_solves_minimum_grid_with_fixed_boundaries() {
        let mut phi = Array2::zeros((3, 3));
        phi[[1, 1]] = 4.0;
        let mut psi = Array2::zeros((3, 3));
        assert!(poisson_relax(&phi, &mut psi, 1.0, false) < 1000);
        assert!((psi[[1, 1]] - 1.0).abs() < 1e-6);
        for ((i, j), &value) in psi.indexed_iter() {
            if i != 1 || j != 1 {
                assert_eq!(value, 0.0);
            }
        }
    }

    #[test]
    fn periodic_poisson_checks_residual_after_updating_ghost_rows() {
        for shape in [(5, 5), (8, 11), (3, 7)] {
            let phi = Array2::zeros(shape);
            let mut psi = Array2::from_shape_fn(shape, |(i, _)| i as f64);
            assert!(poisson_relax(&phi, &mut psi, 1.0, true) < 1000);
            let residual = super::laplace(&psi);
            assert!(residual.iter().all(|value| value.abs() < 1e-6));
            assert_eq!(psi.row(0), psi.row(shape.0 - 2));
            assert_eq!(psi.row(shape.0 - 1), psi.row(1));
        }
    }

    #[test]
    fn periodic_poisson_recovers_manufactured_solution() {
        let (nx, ny) = (10, 9);
        let mut exact = Array2::from_shape_fn((nx, ny), |(i, j)| {
            (std::f64::consts::TAU * i as f64 / (nx - 2) as f64).cos() * (j * (ny - 1 - j)) as f64
        });
        super::sync_periodic_x(&mut exact);
        let h = 0.25;
        let phi = -super::laplace(&exact) / (h * h);
        let mut psi = Array2::zeros((nx, ny));
        assert!(poisson_relax(&phi, &mut psi, h, true) < 1000);
        for ((i, j), value) in psi.indexed_iter() {
            assert!((value - exact[[i, j]]).abs() < 1e-5);
        }
    }

    #[test]
    fn poisson_does_not_report_convergence_for_nan() {
        let phi = Array2::from_elem((3, 3), f64::NAN);
        let mut psi = Array2::zeros((3, 3));
        assert_eq!(poisson_relax(&phi, &mut psi, 1.0, false), 1000);
    }
}
