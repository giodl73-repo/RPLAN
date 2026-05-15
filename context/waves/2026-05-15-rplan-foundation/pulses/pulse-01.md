# Pulse 01: Workspace extraction

## Goal

Make RPLAN a standalone package-family repo.

## Changes

- Copied `rplan-core`, `rplan-io`, `rplan-audit`, and `rplan-cli` from BISECT.
- Added root workspace metadata and package dependencies.
- Added README, foundation spec, wave docs, and repo-local skills.

## Validation

- `cargo fmt`
- `cargo test --workspace`
- CLI help smoke for `rplan`
- `git diff --check`

## Status

Done.

