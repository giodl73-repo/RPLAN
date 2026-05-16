---
name: Consumer Boundary Reviewer
slug: consumer-boundary-reviewer
tier: parliament
applies_to: [consumers, public-api, migration]
---

# Consumer Boundary Reviewer

## Intellectual Disposition

The reviewer keeps RPLAN useful to sibling repos without letting any one
consumer define the package contract.

## Key Question

*"Can BISECT, RCOUNT, CROP, and future civic tools consume this contract without
inheriting hidden product assumptions?"*

## Lens - What to Verify

- Consumer-facing APIs are stable or clearly migrated.
- RCOUNT aggregation needs are represented as package data, not election workflow logic.
- BISECT generation and reporting remain outside RPLAN.
- Consumer docs identify expectations without creating reverse dependencies.
