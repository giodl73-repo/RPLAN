# RPLAN Principles

## RPLAN-P-01: Portable Plan Packages Stay Neutral

**Decision rule:** RPLAN owns district-plan representation, IO, hashes, context,
audit checks, and certificates; generators, maps, reports, research workflows,
and product decisions stay in consumers such as BISECT.

**Rationale:** RPLAN is useful because multiple civic systems can share the
same package contract without inheriting a specific application workflow.

**Test:** Plan Package Steward review and workspace tests preserve the neutral
package boundary.

**Evidence:** `README.md`, `docs/specs/rplan-foundation.md`,
`.roles/parliament/plan-package-steward.md`, and `cargo test --workspace`.

## RPLAN-P-02: Hashes Bind Evidence, Not Narratives

**Decision rule:** Canonical plan, unit-universe, context, source, geometry, and
certificate hashes must bind the exact evidence being verified.

**Rationale:** A district-plan package is auditable only when identifiers,
ordering, provenance, and context changes cannot drift behind stable labels.

**Test:** Core, IO, audit, and CLI tests cover canonical JSON, hash stability,
stale context rejection, source-hash comparison, and certificate verification.

**Evidence:** `docs/compatibility.md`, `crates/rplan-core/src/lib.rs`,
`crates/rplan-io/src/lib.rs`, and `cargo test --workspace`.

## RPLAN-P-03: Certificates Explain Scope

**Decision rule:** An audit certificate may report checks, witnesses, warnings,
missing inputs, profile scope, and verification status, but it must not claim
legal sufficiency, fairness, optimality, or real-data quality beyond the
declared profile and package evidence.

**Rationale:** Civic evidence needs strong machine checks and equally strong
claim boundaries.

**Test:** Audit Certificate Auditor review, certificate disclaimers, public
example docs, and negative fixtures keep success-shaped outputs bounded.

**Evidence:** `.roles/parliament/audit-certificate-auditor.md`,
`docs/examples/u20-plan-audit-certificates/README.md`, and
`docs/examples/u20-plan-audit-certificates/NEGATIVE-FIXTURES.md`.

## RPLAN-P-04: Consumers Own Their Workflows

**Decision rule:** BISECT, RCOUNT, CROP, and future civic tools may consume
RPLAN packages, but their generation, election-count aggregation, indexing,
policy, reporting, and release claims remain downstream-owned.

**Rationale:** A shared foundation should reduce duplicate package logic
without importing hidden consumer assumptions into RPLAN.

**Test:** Consumer Boundary Reviewer review and compatibility rehearsals define
when downstream contracts are affected.

**Evidence:** `.roles/parliament/consumer-boundary-reviewer.md`,
`docs/compatibility.md`, and `context/waves/2026-05-15-rplan-foundation/WAVE.md`.

## RPLAN-P-05: Fixtures Are Cross-Platform Evidence

**Decision rule:** Golden fixtures and CLI tests should compare semantic JSON
contract content, not platform-specific line endings.

**Rationale:** Certificate regressions should fail on meaning, not on whether a
Windows checkout normalizes fixture newlines differently than CLI stdout.

**Test:** Golden-certificate tests parse stdout and fixture JSON before
comparison.

**Evidence:** `crates/rplan-cli/tests/audit_cli.rs` and
`cargo test --workspace`.
