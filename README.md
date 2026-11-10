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
- `ParameterSchema` and `LoggingSchema` derives for machine-readable input and CSV metadata.

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
  src/lib.rs     # Field, ParameterSchema, LoggingSchema derives
```

## Basic Usage

Add the library and `ndarray` to your application:

```toml
[dependencies]
pdifflib = "0.2.1"
ndarray = "0.17.2"
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

The discrete operators assume unit grid spacing. Divide first derivatives by
`h`, and the Laplacian by `h * h`, when working with physical coordinates.
`poisson_relax` accepts `h` explicitly and solves `laplace(psi) = -phi * h * h`.
It returns a zero-based sweep index, or `1000` if convergence was not reached.
Its stopping tolerance is `1e-6` on the absolute discrete residual.

For `periodic = true`, x rows `0` and `nx - 1` are ghost rows copied from
`nx - 2` and `1`; y boundaries remain fixed. `poisson_relax`, `dx_mut`, `dy_mut`,
and `laplace_mut` require matching array dimensions and a grid of at least 3x3.
Invalid dimensions panic before any unchecked access, including in release.
The `_mut` operators overwrite only interior nodes; output boundaries are preserved.
Exported operator macros (`dx!`, `dy!`, `lap!`, `jacobian!`, and the forward,
backward, and averaging variants) borrow their array arguments and evaluate every
array and index expression exactly once.

## Derive Macro

`pdifflib_derive` contains a procedural macro for implementing `pdifflib::field::Field` on structs that store data in a field named `f`.

Add the derive crate when using `Field`:

```toml
[dependencies]
pdifflib = "0.2.1"
pdifflib_derive = "0.2.1"
ndarray = "0.17.2"
```

Example usage:

```rust
use ndarray::Array2;
use pdifflib_derive::Field;

#[derive(Field)]
#[field(name = "vorticity")]
struct Vorticity {
    f: Array2<f64>,
}
```

## Parameter And Logging Schemas

The schema derives are re-exported by `pdifflib`. Every field requires a
description. Parameter fields support `f64`, `usize`, `bool`, and `String`;
logging fields support `f64`, `bool`, and `String`.

```rust
use pdifflib::params::{ParameterSchema as _, LoggingSchema as _};

#[derive(pdifflib::ParameterSchema)]
struct Config {
    #[parameter(description = "Signed offset", minimum = -2.0,
                allowed_values = [-1.0, 0.0, 1.0], default = -1.0)]
    offset: f64,
}

#[derive(pdifflib::LoggingSchema)]
struct Log {
    #[logging(description = "Simulation time")]
    time: f64,
}

let inputs = Config::schema_json("config.toml").expect("finite numeric metadata");
let outputs = Log::schema_json("foo.csv");
assert!(inputs.contains("\"default\": -1"));
assert!(outputs.contains("\"time\""));
```

`minimum`, `exclusive_minimum`, `allowed_values`, and `default` describe numeric
metadata and accept finite integer or floating-point literals, including negative
values. These schemas are descriptive metadata, not automatic input validation
or a complete implementation of the JSON Schema standard.
`ParameterSchema::schema_json` and `input_schema_json` return
`Result<String, SchemaError>` and reject non-finite numeric attributes, including
manually constructed `Parameter` values. The error identifies the parameter and
attribute. `LoggingSchema::schema_json` returns `String` because it only emits
escaped strings and does not serialize numeric attributes.

## System Trait

The `System` trait describes a time-dependent simulation:

```no_run
use csv::Writer;
use pdifflib::field::Field2D;
use pdifflib::system::{SolveResult, System};
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

fn main() -> SolveResult {
    let mut system = MySystem {
        h: 1.0,
        max_time: 2.0,
        dt: 0.01,
        u: Field2D::new("u", 64, 64),
    };
    system.solve(true)?;
    Ok(())
}
```

The loop uses `storage.h5` in the current directory for restart data and appends
logs to `foo.csv`. When `save_map` is enabled, it saves a checkpoint every 50,000
steps and always saves the final state, including short runs. Restart searches
from newest to oldest for readable datasets with matching field shapes and an
exactly matching stored grid spacing `h`. Stage times must be finite and
non-negative. Unreadable or incompatible candidates are skipped in favor of an
older usable stage. `initial_condition` runs only for new storage or storage with
no stages. If stages exist but none is usable, `solve` returns an error without
initializing or overwriting the fields.

`h` is stored as a scalar HDF5 attribute on every new stage and must be finite and
positive. Legacy stages without `h` cannot be used for automatic restart. They
can still be read with `io::read_stage_h5` and explicitly rewritten using
`io::write_stage_h5` with their known original grid spacing. The low-level reader
checks readability but does not require the caller's grid shape or spacing.

When replacing a stage, the writer flushes its new data under `map/.pending`,
moves the old stage to `map/.previous/<stage-name>`, and retains that backup until
the new stage is installed and flushed. Readers can use the backup after an
interrupted replacement; the next write completes recovery before proceeding.
This protocol requires the HDF5 file itself to remain readable; it is not a
filesystem-level atomic transaction.

`solve` returns errors for failed I/O and invalid time steps. `get_dt` must return
a finite positive step that advances the clock; the final step is clipped to the
remaining duration. `get_max_time` is read once at startup and must be finite and
non-negative. User-provided `next_step` is responsible for applying boundary
conditions as required by the numerical scheme.

`System::read_mat` returns `hdf5::Result<f64>` and preserves fields on a failed
restart. `System::write_mat` returns `hdf5::Result<()>`; `System::write_last` and text output
helpers return `std::io::Result<()>`. Handle these results with `?` or explicitly.
`log_params` retains its existing callback signature; custom implementations are
responsible for handling their own write errors. The loop flushes its CSV writer
before returning successfully.

## Publishing

From a release-ready checkout:

- Keep both package versions and the `pdifflib_derive` dependency version in sync,
  and update the dependency examples in both READMEs when changing versions.
- Review `CHANGELOG.md`, especially API changes and checkpoint compatibility.
  After the first publication, breaking API changes require a new minor version
  while the crates are at `0.x`.
- Install the HDF5 development libraries described above.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --release --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo package --workspace
cargo publish --workspace --dry-run
```

Recent Cargo versions can package and dry-run both crates together, resolving
the unpublished workspace dependency through a temporary registry. For actual
publication, publish `pdifflib_derive` first, wait for its matching version to be
available in the crates.io index, then dry-run and publish `pdifflib`. Both
manifests must reference the intended derive version. On older Cargo versions,
verify the derive package individually until it is available in the index.

The GitHub Actions workflow in `.github/workflows/ci.yml` runs the same checks on
Ubuntu, including a publish dry run. It never uploads packages or needs a token.

Once the checks pass, commit the release changes so Cargo packages a clean
checkout. Authenticate locally with `cargo login` (do not commit the token), then:

```bash
cargo publish -p pdifflib_derive
# Wait until crates.io has indexed pdifflib_derive 0.2.1, then:
cargo publish -p pdifflib --dry-run
cargo publish -p pdifflib
```

Publishing is permanent: an uploaded version cannot be replaced. After both
uploads succeed, date the changelog entry and create the release tag.

## License

Licensed under either the MIT License or the Apache License, Version 2.0, at your
option. Both license texts are provided in
[LICENSE](https://github.com/che4web/pdifflib/blob/main/LICENSE) and included in each
published package.

## Notes

- Unsafe pointwise `Field` methods require the caller to satisfy their documented index preconditions. Safe bulk methods validate dimensions before unchecked access.
- `Cargo.lock` is ignored because this repository is currently a library workspace.
- Generated files such as `target/`, `storage.h5`, `foo.csv`, and `res/` should not be committed.
