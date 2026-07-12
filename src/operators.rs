#[macro_export]
macro_rules! dx {
    ($f:expr, $i:expr, $j:expr) => {
        ($f[[$i + 1, $j]] - $f[[$i - 1, $j]]) * 0.5
    };
}

#[macro_export]
macro_rules! dy {
    ($f:expr, $i:expr, $j:expr) => {
        ($f[[$i, $j + 1]] - $f[[$i, $j - 1]]) * 0.5
    };
}

#[macro_export]
macro_rules! dx_f {
    ($f:expr, $i:expr, $j:expr) => {
        $f[[$i + 1, $j]] - $f[[$i, $j]]
    };
}

#[macro_export]
macro_rules! dx_b {
    ($f:expr, $i:expr, $j:expr) => {
        $f[[$i, $j]] - $f[[$i - 1, $j]]
    };
}

#[macro_export]
macro_rules! dy_f {
    ($f:expr, $i:expr, $j:expr) => {
        $f[[$i, $j + 1]] - $f[[$i, $j]]
    };
}

#[macro_export]
macro_rules! dy_b {
    ($f:expr, $i:expr, $j:expr) => {
        $f[[$i, $j]] - $f[[$i, $j - 1]]
    };
}

#[macro_export]
macro_rules! mx_b {
    ($f:expr, $i:expr, $j:expr) => {
        ($f[[$i, $j]] + $f[[$i - 1, $j]]) * 0.5
    };
}

#[macro_export]
macro_rules! my_b {
    ($f:expr, $i:expr, $j:expr) => {
        ($f[[$i, $j]] + $f[[$i, $j - 1]]) * 0.5
    };
}

#[macro_export]
macro_rules! lap {
    ($f:expr, $i:expr, $j:expr) => {
        $f[[$i + 1, $j]] + $f[[$i - 1, $j]] + $f[[$i, $j + 1]] + $f[[$i, $j - 1]]
            - 4.0 * $f[[$i, $j]]
    };
}

#[macro_export]
macro_rules! jacobian {
    ($v:expr, $t:expr, $i:expr, $j:expr) => {
        $crate::dx!($v, $i, $j) * $crate::dy!($t, $i, $j)
            - $crate::dy!($v, $i, $j) * $crate::dx!($t, $i, $j)
    };
}
