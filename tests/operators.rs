use ndarray::Array2;
use std::cell::Cell;

fn counted<T, const N: usize>(calls: &Cell<[usize; N]>, index: usize, value: T) -> T {
    let mut counts = calls.get();
    counts[index] += 1;
    calls.set(counts);
    value
}

#[test]
fn operators_evaluate_each_argument_once_and_compute_expected_values() {
    let field = Array2::from_shape_fn((5, 6), |(i, j)| (i * i + 3 * j * j) as f64);
    macro_rules! check {
        ($operator:ident, $expected:expr) => {{
            let calls = Cell::new([0; 3]);
            let result = pdifflib::$operator!(
                counted(&calls, 0, &field),
                counted(&calls, 1, 2usize),
                counted(&calls, 2, 2usize)
            );
            assert_eq!(result, $expected, stringify!($operator));
            assert_eq!(calls.get(), [1, 1, 1], stringify!($operator));
        }};
    }
    check!(dx, 4.0);
    check!(dy, 12.0);
    check!(dx_f, 5.0);
    check!(dx_b, 3.0);
    check!(dy_f, 15.0);
    check!(dy_b, 9.0);
    check!(mx_b, 14.5);
    check!(my_b, 11.5);
    check!(lap, 8.0);

    let other = Array2::from_shape_fn(field.dim(), |(i, j)| (2 * i + 5 * j) as f64);
    let calls = Cell::new([0; 4]);
    let jacobian = pdifflib::jacobian!(
        counted(&calls, 0, &field),
        counted(&calls, 1, &other),
        counted(&calls, 2, 2usize),
        counted(&calls, 3, 2usize)
    );
    assert_eq!(jacobian, -4.0);
    assert_eq!(calls.get(), [1; 4]);
}

#[test]
fn index_side_effects_do_not_change_the_stencil_mid_evaluation() {
    let field = Array2::from_shape_fn((6, 5), |(i, _)| (i * i) as f64);
    let mut i = 1;
    assert_eq!(
        pdifflib::dx!(
            field,
            {
                i += 1;
                i
            },
            2
        ),
        4.0
    );
    assert_eq!(i, 2);
    // The macro borrows owned inputs instead of moving them and accepts views
    // and temporary arrays as well as references.
    assert_eq!(pdifflib::dx!(field.view(), 2, 2), 4.0);
    assert_eq!(pdifflib::dx!(field.clone(), 2, 2), 4.0);
}
