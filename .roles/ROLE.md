# RPLAN Role Index

RPLAN owns portable district-plan packages, IO, audit checks, and CLI contracts.
Use these roles when changing plan models, package formats, audit certificates,
fixtures, or downstream consumer boundaries.

## Parliament

| File | Role | Primary tension |
|---|---|---|
| `parliament/plan-package-steward.md` | Plan Package Steward | Portable representation vs. BISECT workflow leakage |
| `parliament/audit-certificate-auditor.md` | Audit Certificate Auditor | Reproducible validation vs. optimistic package acceptance |
| `parliament/consumer-boundary-reviewer.md` | Consumer Boundary Reviewer | Shared civic package contracts vs. downstream coupling |

## Review order

1. Use Plan Package Steward for model, hashing, IO, and format changes.
2. Use Audit Certificate Auditor for validation checks, certificates, and negative fixtures.
3. Use Consumer Boundary Reviewer when BISECT, RCOUNT, CROP, or future consumers are affected.
