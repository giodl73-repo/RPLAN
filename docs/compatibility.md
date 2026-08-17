# RPLAN compatibility policy

RPLAN is a pre-1.0 shared civic-evidence foundation. Compatibility is
deliberate because downstream systems depend on its plan packages, context
records, canonical hashes, audit certificates, and versioned IO behavior.

## Protected contract

The protected surface includes:

- public APIs and error meanings from `rplan-core`, `rplan-io`, and
  `rplan-audit`;
- `district-plan-v1`, RPLAN `0.2` and retained `0.1` conversion behavior,
  RCTX `0.1`, `audit-certificate-v1`, and `legal-profile-v1`;
- plan-unit identity, canonical ordering, assignment validation, district
  ranges, labels, and empty-district policy;
- canonical JSON, plan hashes, unit-universe hashes, context hashes, source
  hashes, and geometry hashes;
- RCTX graph, population, subdivision, demographic, geometry, and source
  provenance validation;
- read/write round-tripping, extension preservation, and stale-hash rejection;
- audit finding, certificate, profile, and certificate-verification meanings;
  and
- the boundary that consumers own plan generation, election counts, maps,
  research workflows, and domain policy.

Internal refactoring is compatible only when these observable contracts remain
stable.

## Versioning rules

- Additive APIs or optional package fields may remain within the current `0.y`
  line when existing packages, hashes, and consumers remain compatible.
- Breaking APIs, validation, error meanings, canonical ordering, hash inputs,
  package fields, defaults, or audit semantics require a minor-version bump
  while the affected crate is below `1.0`.
- A breaking serialized record requires a new schema or package version rather
  than silently changing an existing version.
- Readers should retain an explicit migration path for supported older package
  versions; writers emit the current version.
- Prefer deprecation plus migration notes before removing a public item.
- A breaking change must identify affected consumers and include package,
  context, hash, or certificate migration guidance.
- Downstream repositories should pin commits for reproducible evidence.
  Branch consumers must run the downstream rehearsal before updating.

## Foundation tests

From the RPLAN repository:

```powershell
cargo test -p rplan-core -p rplan-io
cargo test -p rplan-audit
cargo test -p rplan-cli
```

These protect canonical hashing, plan and context validation, versioned IO,
fixture round-tripping, stale-hash rejection, audit behavior, certificates, and
CLI package handling.

## Downstream breakage rehearsal

RCOUNT is the required first consumer rehearsal because district aggregation
reads RPLAN packages, validates plans and optional RCTX references, uses
canonical plan hashes, and records those hashes in count transcripts.

Use `RCOUNT\repo-map.toml` to create the ignored local Cargo patch for sibling
RPLAN and RLINE checkouts. Then run from RCOUNT:

```powershell
cargo test -p rcount-district aggregates_summary_basic_into_rplan_districts
cargo test -p rcount-district aggregation_transcript_records_matching_rctx_reference
cargo test -p rcount-district aggregation_rejects_declared_crosswalk_hash_drift
```

A compile failure exposes public API breakage. Plan, context, hash, transcript,
or district-total failures expose serialized or behavioral drift.

RPLAN foundation changes are not ready until the affected foundation tests and
the RCOUNT rehearsal pass.
