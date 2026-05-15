# RPLAN Foundation Spec

## Goal

Extract the generic district-plan package family from BISECT into a neutral
workspace that downstream tools can consume directly.

## Initial crates

| Crate | Role |
|-------|------|
| `rplan-core` | plan/context data model and canonical hashes |
| `rplan-io` | package read/write and legacy format migration |
| `rplan-audit` | package audit checks and certificates |
| `rplan-cli` | standalone command-line surface |

## Boundary

RPLAN owns plan-package representation and validation. It does not own
redistricting algorithms, plan generation, map rendering, ensemble search,
district-count workflows, or BISECT report orchestration.

## Consumers

- BISECT: generates and audits district plans.
- RCOUNT: can aggregate election counts against RPLAN assignments.
- CROP: may index RPLAN docs or package side-info.

