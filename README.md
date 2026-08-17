# RPLAN

**Reusable district-plan package, IO, audit, and CLI contracts.**

**Series:** [Election Systems](https://github.com/giodl73-repo/giodl73-repo/blob/main/series/election-systems.md).

**Review roles:** This repo uses
[ROLES](https://github.com/giodl73-repo/ROLES), the `.roles` convention for
repository-local review panels.

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
