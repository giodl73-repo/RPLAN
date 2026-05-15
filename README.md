# RPLAN

**Reusable district-plan package, IO, audit, and CLI contracts.**

**Series:** [Election Systems](https://github.com/giodl73-repo/giodl73-repo/blob/main/series/election-systems.md).

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

- [`docs\specs\rplan-foundation.md`](docs/specs/rplan-foundation.md) records the
  extraction boundary.
- `context\waves\` tracks implementation waves and pulse history.

## License

MIT. See [`LICENSE`](LICENSE).

