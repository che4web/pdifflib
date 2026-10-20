# pdifflib_derive

Procedural macros for [pdifflib](https://crates.io/crates/pdifflib):

- `Field`: implements `pdifflib::field::Field` for structs with an
  `f: ndarray::Array2<f64>` field. Override the field name with
  `#[field(name = "temperature")]`.
- `ParameterSchema`: describes input parameters annotated with
  `#[parameter(description = "...")]`. Numeric metadata supports `minimum`,
  `exclusive_minimum`, `allowed_values`, and `default`, including negative values.
- `LoggingSchema`: describes CSV columns annotated with
  `#[logging(description = "...")]`.

```toml
[dependencies]
pdifflib = "0.2.1"
pdifflib_derive = "0.2.1"
ndarray = "0.17.2"
```

`ParameterSchema` and `LoggingSchema` are also re-exported by `pdifflib`, so a
direct dependency on this crate is only needed for `Field`. See the
[pdifflib documentation](https://docs.rs/pdifflib) for compiled usage examples,
supported field types, and simulation helpers.

The generated implementations reference `::pdifflib` and, for `Field`, `ndarray`.
Keep these dependency names when using the macros.

## License

MIT OR Apache-2.0, at your option. Both license texts are included in `LICENSE`.
