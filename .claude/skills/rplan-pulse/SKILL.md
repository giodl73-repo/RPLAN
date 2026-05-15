---
name: rplan-pulse
description: Execute the next RPLAN wave pulse with docs, implementation, validation, and commit-ready updates.
allowed-tools:
  - Read
  - Write
  - Glob
  - Grep
  - Bash
---

# RPLAN Pulse

Use this skill for RPLAN development pulses.

## Workflow

1. Read `context/waves/PHASES.md`.
2. Read the active wave `WAVE.md`.
3. Read the target pulse under `pulses\`.
4. Implement the smallest complete generic plan-package slice.
5. Keep BISECT product behavior out of RPLAN.
6. Update docs and wave/pulse status.
7. Run `cargo fmt`, `cargo test --workspace`, focused CLI smokes, and
   `git diff --check`.

