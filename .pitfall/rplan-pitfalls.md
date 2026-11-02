# RPLAN Pitfalls

## RPLAN-PF-01: Certificate Pass Becomes Legal Approval

**Status:** MITIGATED

**Pattern:** A passing RPLAN audit certificate is presented as legal
certification, fairness proof, public release approval, or full redistricting
acceptance instead of a scoped profile/package verification result.

**Domain:** Public examples, customer-facing civic evidence, BISECT reports,
and future downstream release workflows.

**Actor:** Civic-tool maintainer, public-report author, customer reviewer,
BISECT consumer, or future release agent.

**Task:** Present an RPLAN audit certificate or example package without turning
profile-scoped verification into legal, fairness, public-release, or
redistricting acceptance authority.

**Surface:** README, U.20 audit-certificate examples, golden packages,
benchmark package docs, certificate CLI output, and role review.

**Likely mistake:** Read `pass`, certificate verification, coherent hashes, or
tiny synthetic examples as legal approval, fairness proof, large-instance
quality, policy preference, or release approval.

**Consequence:** Downstream reports can overclaim civic authority and make
customer/public decisions depend on a package verifier that only proves scoped
format, hash, context, and profile semantics.

**Owner:** RPLAN audit-certificate auditor and the downstream repo publishing
the civic claim.

**Detection difficulty:** The word `pass` is operationally useful but easy to
over-read outside the exact check/profile context.

**Structural solution:** Keep the authority-boundary contract, certificate
disclaimers, role review, and consumer-owned release language visible in every
public promotion path.

**Evidence:** `docs/examples/u20-plan-audit-certificates/README.md`,
`docs/examples/rplan-golden-packages/README.md`,
`docs/examples/rplan-benchmark-packages/README.md`,
`docs/authority-boundaries.v1.json`, and
`.roles/parliament/audit-certificate-auditor.md`.

**Test:** `cargo test -p rplan-cli --test pitfall_policy`.

## RPLAN-PF-02: Consumer Pending Work Becomes Adopted Contract

**Status:** MITIGATED

**Pattern:** Pending BISECT dependency rewire or RCOUNT integration work is
treated as already adopted because RPLAN has a standalone workspace and public
compatibility policy.

**Domain:** Portfolio dependency adoption, BISECT/RCOUNT integration,
submodule snapshots, and customer readiness scoring.

**Actor:** Portfolio maintainer, BISECT/RCOUNT integrator, dependency upgrader,
or customer-readiness reviewer.

**Task:** Assess whether RPLAN's standalone workspace and compatibility policy
mean downstream dependency rewires have been completed.

**Surface:** Foundation wave, compatibility policy, README family diagram,
TRACKER submodule reports, and consumer migration notes.

**Likely mistake:** Treat RPLAN-local build/test success, public compatibility
docs, or a submodule pointer update as proof that BISECT and RCOUNT have
adopted the dependency change.

**Consequence:** Portfolio status can mark a shared civic contract as adopted
while downstream repos still own pending rewires, rehearsals, or validation
failures.

**Owner:** RPLAN consumer-boundary reviewer plus the affected BISECT/RCOUNT
repo owners.

**Detection difficulty:** RPLAN has working crates and examples, but the active
wave still marks consumer rewires pending.

**Structural solution:** Keep the authority-boundary contract and pending
pulses visible until consumer repos own the dependency change and their
validation passes.

**Evidence:** `context/waves/2026-05-15-rplan-foundation/WAVE.md`,
`docs/compatibility.md`, `docs/authority-boundaries.v1.json`, and
`.roles/parliament/consumer-boundary-reviewer.md`.

**Test:** `cargo test -p rplan-cli --test pitfall_policy`.

## RPLAN-PF-03: RCOUNT Rehearsal Is Skipped

**Status:** MITIGATED

**Pattern:** A change to public APIs, hashes, package formats, or audit
semantics is considered ready after RPLAN-local tests only, without the required
RCOUNT downstream rehearsal.

**Domain:** Shared civic package compatibility, district aggregation,
transcripts, and portfolio dependency updates.

**Actor:** RPLAN maintainer, public API editor, package-format editor, audit
semantics reviewer, or release agent.

**Task:** Decide whether a change that affects APIs, hashes, package formats, or
audit semantics is ready for release or portfolio pointer promotion.

**Surface:** Compatibility policy, RCOUNT golden-package docs, RPLAN/RCOUNT
rehearsal commands, audit fixtures, and package-format docs.

**Likely mistake:** Stop after broad RPLAN-local tests and skip the first
consumer rehearsal because no local package or certificate test failed.

**Consequence:** RCOUNT district aggregation, transcripts, package hashes, or
verification behavior can break after the RPLAN pointer is promoted.

**Owner:** RPLAN consumer-boundary reviewer and RCOUNT integration owner.

**Detection difficulty:** RPLAN-local tests are broad, so downstream breakage
can look unlikely until RCOUNT compiles or verifies real package bridges.

**Structural solution:** Treat the RCOUNT rehearsal in
`docs/authority-boundaries.v1.json` and `docs/compatibility.md` as a release
gate for affected foundation changes.

**Evidence:** `docs/authority-boundaries.v1.json`, `docs/compatibility.md`, and
`docs/examples/rcount-golden-packages/README.md`.

**Test:** `cargo test -p rplan-cli --test pitfall_policy`.

## RPLAN-PF-04: Raw Golden Text Becomes Cross-Platform Proof

**Status:** MITIGATED

**Pattern:** Golden-certificate tests compare raw pretty-printed JSON strings,
so CRLF/LF differences fail tests even when certificate content is identical.

**Domain:** Windows validation, CLI certificate tests, fixture maintenance, and
research-paper proof replay.

**Detection difficulty:** The failure appears as a certificate mismatch, but
the only difference is platform newline representation.

**Structural solution:** Parse stdout and fixture JSON before comparing golden
certificate values.

**Evidence:** `crates/rplan-cli/tests/audit_cli.rs` and
`cargo test --workspace`.

## RPLAN-PF-05: Algorithm Lineage Overrides Certificate Truth

**Status:** MITIGATED

**Pattern:** Method-produced package lineage includes reserved fields that
shadow or override top-level certificate fields such as `plan_hash`,
`source_hashes`, or `result`.

**Domain:** BISECT-generated method packages, benchmark packages, audit
certificates, and cross-repo research artifacts.

**Detection difficulty:** Lineage extras are intentionally flexible, so
reserved field misuse can look like harmless method metadata.

**Structural solution:** Reject reserved lineage fields in audit and CLI tests.

**Evidence:** `crates/rplan-audit/src/lib.rs`,
`crates/rplan-cli/tests/audit_cli.rs`, and
`docs/examples/u20-plan-audit-certificates/NEGATIVE-FIXTURES.md`.
