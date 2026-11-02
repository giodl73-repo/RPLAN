# Pulse 04: PITFALL use-case integration

## Result

Added use-case-first fields and retained policy-test coverage for RPLAN's three
open certificate, consumer-adoption, and downstream-rehearsal pitfalls.

## Covered pitfalls

- `RPLAN-PF-01` keeps a passing audit certificate scoped to package/profile
  verification rather than legal approval, fairness proof, public release, or
  redistricting acceptance.
- `RPLAN-PF-02` keeps pending BISECT and RCOUNT dependency rewires visible until
  the consuming repos own and validate the adoption.
- `RPLAN-PF-03` keeps RCOUNT rehearsal as a release gate for affected RPLAN
  public API, hash, package-format, or audit-semantics changes.

## Validation

```powershell
cargo fmt --check
cargo test -p rplan-cli --test pitfall_policy
cargo test --workspace
cargo run -p rplan-cli -- --help
```
