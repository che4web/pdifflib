# pdifflib

`pdifflib` is a Rust library for finite-difference computations on two-dimensional fields. It provides basic field containers, grid iterators, differential operators, simple iterative solvers, and helpers for writing simulation data.

The repository is organized as a Cargo workspace with two crates:

```text
pdifflib/          # main library
pdifflib_derive/   # procedural macros for pdifflib
```

## Features

- `Field2D` container based on `ndarray::Array2<f64>`.
- `Grid` iterators for inner nodes and cell faces.
- Finite-difference operators: `dx`, `dy`, forward/backward differences, Laplace operator, and averaging helpers.
- `System` trait with a built-in simulation loop, periodic checkpoint reading, and data writing.
- HDF5, CSV, and text output helpers.
- `pdifflib_derive::Field` derive macro for implementing the `Field` trait on custom field structs.

## Requirements

- Rust 2021 edition.
- Cargo.
- HDF5 development libraries available on the system, required by `hdf5-metno`.

On Debian/Ubuntu-like systems this is usually:

```bash
sudo apt install libhdf5-dev
```

## Build And Test

```bash
cargo check --workspace
cargo test --workspace
```

## Workspace Layout

```text
Cargo.toml
src/
  field.rs       # Field2D, Grid, Field trait
  finit_diff.rs  # finite-difference functions
  io.rs          # text and HDF5 output/input helpers
  operators.rs   # exported operator macros
  system.rs      # System trait and solve loop
pdifflib_derive/
  Cargo.toml
  src/lib.rs     # #[derive(Field)]
```

## Basic Usage

Add the library to another local crate:

```toml
[dependencies]
pdifflib = { path = "../pdifflib" }
```

Create a field and iterate over inner grid nodes:

```rust
use pdifflib::field::{Field2D, Grid};

let mut temperature = Field2D::new("temperature", 128, 128);
let grid = Grid::new(128, 128);

for (i, j) in grid.inner_nodes() {
    temperature.f[[i, j]] = 1.0;
}
```

Use finite-difference helpers:

```rust
use ndarray::Array2;
use pdifflib::finit_diff::{dx, dy, laplace};

let field = Array2::<f64>::zeros((64, 64));
let grad_x = dx(&field);
let grad_y = dy(&field);
let lap = laplace(&field);
```

Solve a Poisson-like relaxation problem:

```rust
use ndarray::Array2;
use pdifflib::finit_diff::poisson_relax;

let phi = Array2::<f64>::zeros((64, 64));
let mut psi = Array2::<f64>::zeros((64, 64));
let iterations = poisson_relax(&phi, &mut psi, 1.0, false);
```

## Derive Macro

`pdifflib_derive` contains a procedural macro for implementing `pdifflib::field::Field` on structs that store data in a field named `f`.

Example dependency for another local crate:

```toml
[dependencies]
pdifflib = { path = "../pdifflib" }
pdifflib_derive = { path = "../pdifflib/pdifflib_derive" }
ndarray = "0.17.2"
```

Example usage:

```rust
use ndarray::Array2;
use pdifflib::field::Field;
use pdifflib_derive::Field;

#[derive(Field)]
#[field(name = "vorticity")]
struct Vorticity {
    f: Array2<f64>,
}
```

## System Trait

The `System` trait describes a time-dependent simulation:

```rust
use csv::Writer;
use pdifflib::field::Field2D;
use pdifflib::system::System;
use std::fs::File;

struct MySystem {
    h: f64,
    max_time: f64,
    dt: f64,
    u: Field2D,
}

impl System for MySystem {
    fn next_step(&mut self, dt: f64, time: f64) {
        let _ = (dt, time);
    }

    fn fields(&self) -> Vec<&Field2D> {
        vec![&self.u]
    }

    fn fields_mut(&mut self) -> Vec<&mut Field2D> {
        vec![&mut self.u]
    }

    fn get_h(&self) -> f64 {
        self.h
    }

    fn boundary_condition(&mut self) {}

    fn get_max_time(&self) -> f64 {
        self.max_time
    }

    fn log_params(&self, _wrt: &mut Writer<File>, _time: f64) {}

    fn initial_condition(&mut self) {}

    fn get_dt(&self) -> f64 {
        self.dt
    }
}
```

Run the built-in loop with:

```rust
system.solve(true);
```

When `save_map` is enabled, the loop writes HDF5 stages and uses `storage.h5` for restart data.

## Notes

- Many low-level finite-difference methods use unchecked indexing internally for speed. Call them only on arrays with valid sizes and indices.
- `Cargo.lock` is ignored because this repository is currently a library workspace.
- Generated files such as `target/`, `storage.h5`, `foo.csv`, and `res/` should not be committed.
