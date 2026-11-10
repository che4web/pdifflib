# Changelog

## 0.2.1 (unreleased)

### Fixed

- Validate array shapes and minimum grid dimensions before unchecked bulk access.
- Synchronize periodic x boundaries and check the Poisson residual before
  declaring convergence; use up-to-date periodic neighbors during SOR sweeps.
- Do not report convergence for non-finite Poisson updates.
- Always save the final simulation checkpoint when `save_map` is enabled.
- Return errors for invalid/non-advancing time steps and non-finite time limits.
- Return HDF5 opening errors instead of panicking; read checkpoints read-only.
- Ignore non-finite or negative restart timestamps and stage incomplete HDF5
  writes under a name that cannot be selected for restart.
- Accept signed numeric literals in parameter schema attributes.
- Preserve generics in derive implementations and diagnose malformed `Field`
  attributes instead of ignoring parse errors.
- Search older checkpoint candidates when newer datasets cannot be read or do
  not match the current grid. Return an error instead of silently initializing
  when existing stages are unusable.
- Persist and validate checkpoint grid spacing. Legacy stages without spacing
  metadata require explicit conversion before automatic restart.
- Retain a readable replacement backup until the new checkpoint is installed
  and flushed; recover interrupted replacements on subsequent reads/writes.
- Reject non-finite attributes in input JSON generation, including parameters
  constructed without the derive macro.
- Evaluate all operator macro arguments exactly once.

### API changes

- `System::solve` now returns `SolveResult`.
- `System::write_mat` now returns `hdf5::Result<()>`.
- `System::read_mat` now returns `hdf5::Result<f64>`.
- `System::write_last`, `io::write_mat`, and `io::write_stage` now return
  `std::io::Result<()>`.
- Callers and overriding trait methods must handle or return these results.
- `ParameterSchema::schema_json` and `input_schema_json` now return
  `Result<String, SchemaError>`.

### Packaging

- Add CI checks for formatting, linting, debug/release tests, documentation, and
  a workspace publish dry run; document authentication and publication order.
- Add package metadata and MIT OR Apache-2.0 licensing for both crates.
- Compile the main README examples as rustdoc tests.
- Remove unused direct dependencies on nalgebra, ndarray-stats, queues, serde,
  and toml.
