# Wave: RPLAN Foundation

## Goal

Create a standalone Rust workspace for the reusable RPLAN package family.

## Pulse table

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | Workspace extraction | done | Copied RPLAN crates from BISECT, added standalone workspace metadata, docs, and validation. |
| 02 | BISECT dependency rewire | pending | Update BISECT to consume RPLAN from the sibling repo or git dependency. |
| 03 | RCOUNT integration | pending | Keep RCOUNT district aggregation depending on RPLAN without a BISECT workspace dependency. |

## Success criteria

- RPLAN has its own Rust workspace and git repo.
- Existing RPLAN crates build and test outside BISECT.
- Docs define product boundaries and consumers.
- `cargo fmt`, `cargo test --workspace`, CLI help smoke, and `git diff --check`
  pass.

