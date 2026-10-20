# Remaining publication review tasks

The following P2 issues were reproduced during the release review and remain
open. Fixes already implemented are listed in [CHANGELOG.md](CHANGELOG.md).

- [ ] **Define CSV logging behavior on restart.**
  - Location: `src/system.rs` (`solve`, opening and appending `foo.csv`).
  - Reproduction: save a checkpoint at `t=2`, continue to `t=4` without saving
    checkpoints, then restart and run to `t=5`. Log timestamps become
    `0, 2, 2, 4, 2, 5` because records newer than the restored checkpoint remain.
  - Choose an explicit restart policy, such as a separate log per run. Do not
    assume that the first column of arbitrary `log_params` output is time.
  - Add an integration test covering rollback to an older checkpoint and the
    chosen log policy; document how repeated checkpoint timestamps are handled.

- [ ] **Support large integer schema attributes without overflow or rounding.**
  - Locations: `pdifflib_derive/src/lib.rs` (numeric attribute expansion) and
    `src/params.rs` (`Parameter` numeric metadata).
  - Reproduction: `default = 4_294_967_295` on a `usize` field fails to compile
    because the generated cast leaves the unsuffixed literal inferred as `i32`.
  - On a 64-bit target, `default = 9_007_199_254_740_993usize` compiles but emits
    `9007199254740992` in JSON because all numeric metadata is stored as `f64`.
  - Preserve integer values separately from floating-point values and remove
    unintended `i32` inference from generated attributes.
  - Test large unsuffixed literals and exact JSON serialization above `2^53`,
    including numeric bounds and allowed values. Document any API changes.

- [ ] **Validate grid spacing in direct `poisson_relax` calls.**
  - Location: `src/finit_diff.rs` (`poisson_relax`, before computing/using `h²`).
  - Reproduction: a 3x3 grid with `phi = 4`, `psi = 0`, and `h = 0` reports
    convergence at sweep zero because the source is multiplied by zero.
  - Require finite positive `h` and finite positive `h²`, rejecting underflow to
    zero and overflow to infinity before mutating the solution.
  - Keep checks outside the numerical loops. Add debug/release tests for zero,
    negative, NaN, infinite, underflowing, and overflowing spacing, along with a
    valid non-unit-spacing case. Document the failure behavior.
