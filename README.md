# RPLAN

**Reusable district-plan package, IO, audit, and CLI contracts.**

**Series:** [Election Systems](https://github.com/giodl73-repo/giodl73-repo/blob/main/series/election-systems.md).

**Review roles:** This repo uses
[ROLES](https://github.com/giodl73-repo/ROLES), the `.roles` convention for
repository-local review panels.

## Inspect a supplied plan in the browser

[Open the plan workbench](https://giodl73-repo.github.io/RPLAN/) to open/edit
RPLAN plans and optional RCTX context, supply a profile, select native audit
constraints and inspect every check, warning and witness. You can verify a
supplied certificate and export original bytes, current/accepted requests,
complete native reports, exact certificates and normalized plan/context files.
The connected/disconnected synthetic examples work without an account or CLI.

A native pass is not legal approval, fairness, data authenticity, public-release
approval or consumer adoption. Profiles, source hashes, lineage and timestamps
remain caller assertions. Certificate verification checks declared identities;
it does not rerun an audit. See [native behavior and limits](browser/plan-workbench/MODEL.md),
[build/source replay](browser/plan-workbench/BUILD.md),
[complete source scope](browser/plan-workbench/SOURCE-SCOPE.md) and the
[browser qualification wave](context/waves/2026-11-02-plan-wasm/WAVE.md).
Implementation, hosted/default deployment and live qualification are separate gates.

## R package family

RPLAN is the district-plan package layer in a reusable civic-evidence family:

```text
                    ┌→ RPLAN  — district-plan packages, IO, and audits ─┐
RLINE — kernels ────┤                                                   ├→ BISECT
                    └→ RCOUNT — count packages and audit replay ────────┘
```

| Repo | Responsibility |
|------|----------------|
| [RLINE](https://github.com/giodl73-repo/RLINE) | Product-neutral graph, context, statistics, optimization, and history kernels. |
| **RPLAN** | Portable district-plan representation, interchange, hashing, and audit certificates. |
| [RCOUNT](https://github.com/giodl73-repo/RCOUNT) | Election-count package verification, reconciliation, aggregation, and audit replay. |
| [BISECT](https://github.com/giodl73-repo/BISECT) | Redistricting application and research workbench that consumes the reusable layers. |

The dependency direction is one-way: portable plan packages do not inherit
BISECT generation, mapping, reporting, or research workflows.

RPLAN is the neutral home for district-plan interchange crates that should be
usable by BISECT, RCOUNT, CROP, and other civic tools without depending on the
BISECT application workspace.

## Workspace

| Crate | Purpose |
|-------|---------|
| `rplan-core` | Generic district-plan domain model, context model, and canonical hashing. |
| `rplan-io` | RPLAN v0.2/v0.1 read-write and context serialization. |
| `rplan-audit` | Generic plan audit checks and certificate generation. |
| `rplan-cli` | `rplan` command-line tools. |

## Design rule

RPLAN owns the portable plan package boundary. Redistricting generation,
algorithm search, maps, reports, and BISECT product workflows stay outside
RPLAN.

## Commands

```powershell
cargo test --workspace
cargo run -p rplan-cli -- --help
```

## Specs

- [`docs\compatibility.md`](docs/compatibility.md) defines the protected
  pre-1.0 package, hash, audit, and IO contracts plus the mandatory RCOUNT
  downstream rehearsal.
- [`docs\specs\rplan-foundation.md`](docs/specs/rplan-foundation.md) records the
  extraction boundary.
- `context\waves\` tracks implementation waves and pulse history.

## License

MIT. See [`LICENSE`](LICENSE).
