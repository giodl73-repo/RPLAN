# RPLAN Invariants

## RPLAN-INV-01: Package Boundary Excludes BISECT Product Logic

**Status:** MITIGATED

**Claim:** RPLAN models, IO, audit checks, and CLI contracts remain reusable
without BISECT generation, mapping, reporting, or research-workbench logic.

**Why it matters:** Consumer leakage would make RPLAN less reusable and would
couple shared civic evidence to one application.

**Enforcement:** README, foundation spec, role review, and workspace tests
preserve the extraction boundary.

**Evidence:** `README.md`, `docs/specs/rplan-foundation.md`,
`.roles/parliament/plan-package-steward.md`, and `cargo test --workspace`.

## RPLAN-INV-02: Context Drift Invalidates Certificates

**Status:** MITIGATED

**Claim:** A certificate generated with context evidence cannot verify against
missing, tampered, stale, or reordered context.

**Why it matters:** Source labels and unit lists can remain plausible while
population, graph, source, or unit-order evidence changes.

**Enforcement:** CLI tests and negative fixture docs cover missing context,
context hash mismatch, stale context with unchanged source label, and unit-order
mismatch.

**Evidence:** `crates/rplan-cli/tests/audit_cli.rs`,
`docs/examples/u20-plan-audit-certificates/NEGATIVE-FIXTURES.md`, and
`cargo test --workspace`.

## RPLAN-INV-03: Reserved Certificate Fields Cannot Be Overridden

**Status:** MITIGATED

**Claim:** Algorithm lineage metadata cannot override top-level certificate
fields such as plan hash, source hashes, result, or certificate identity.

**Why it matters:** Method provenance should explain how a package was produced,
not rewrite what the certificate verifies.

**Enforcement:** Audit library and CLI tests reject reserved lineage fields.

**Evidence:** `crates/rplan-audit/src/lib.rs`,
`crates/rplan-cli/tests/audit_cli.rs`, and
`docs/examples/u20-plan-audit-certificates/NEGATIVE-FIXTURES.md`.

## RPLAN-INV-04: Golden Certificates Are Meaning-Stable Across Platforms

**Status:** MITIGATED

**Claim:** Golden-certificate tests compare parsed JSON values so line-ending
differences do not masquerade as certificate drift.

**Why it matters:** The Windows validation failure showed that raw string
comparison could block a valid proof path while teaching the wrong lesson about
certificate stability.

**Enforcement:** `assert_json_matches_fixture` parses CLI stdout and fixture
content before equality checks.

**Evidence:** `crates/rplan-cli/tests/audit_cli.rs` and
`cargo test --workspace`.

## RPLAN-INV-05: Downstream Rehearsal Gates Foundation Changes

**Status:** MITIGATED

**Claim:** A foundation change is not ready until affected RPLAN tests and the
required RCOUNT rehearsal pass or the rehearsal is explicitly scoped out.

**Why it matters:** RCOUNT consumes RPLAN plan hashes, RCTX references, and
district assignments; local RPLAN green tests alone do not prove consumer
compatibility.

**Enforcement:** Compatibility policy names the RCOUNT rehearsal and expected
failure signals.

**Evidence:** `docs/compatibility.md` and
`.roles/parliament/consumer-boundary-reviewer.md`.
