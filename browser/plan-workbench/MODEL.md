# Native model and caller boundaries

This product calls the existing RPLAN readers, core hashes and audit/certificate
functions. The native CLI and browser share the original chamber/profile
applicability checks. Browser auditing requires an explicit supplied profile;
this is a policy assertion, not a statement of applicable law. The optional
legacy0.1 reader converts into the native0.2 model and normalized export.

Plan-shape always runs. Other checks follow the requested native constraints.
The report preserves pass, fail, pass-with-warnings, missing-input and
not-evaluated outcomes and every witness. Native overall pass may coexist with
not-evaluated checks. The native validator generally trusts the declared
unit-universe hash; a separately computed identity is informational. Caller
schema strings, source hashes, geometry references and provenance are not
authenticated or fetched. A report is not legal approval, fairness, verified
census data, public-release approval or consumer adoption.

Certificate verification checks native content, identifier, plan/context and
source-hash consistency. It does not rerun the audit, authenticate signatures
or prove that checks were executed. Native content hashing omits generated-at,
certificate-id and content-hash fields. Generated-at is a caller label, not an
attested wall clock. Companion runtime labels identify this software, not the
original CLI. Lineage uses the original native constructor and reserved-field
guard; it remains a caller assertion.

Native readers can ignore unknown fields, collapse duplicate object keys and
convert legacy input. Keep original input bytes, current text and normalized
known fields separate. Complete native report and certificate exports must use
the exact Rust-produced text. JavaScript numeric display can round integers
above2^53; it must not reserialize certificates or raw reports for download.

The companion refuses whole operations before native work at1MiB raw request,
256KiB plan/context/certificate,32KiB profile/lineage,768KiB aggregate text,
2048 units,128 districts,8192 graph edges/array entries,65536 JSON nodes and
16KiB scalar strings/object keys. Native serde's recursion bound remains active.
Claimed contiguity/split/geometry witness counts are capped at2048 before
platform-sized integer decoding. Arbitrary lineage/extension integers remain
exact in Rust-produced exports, including values above JavaScript precision.
Population magnitude is at most10^12 so native i64 arithmetic stays in range.
Native auditing assumes compatible indexed arrays; unequal plan/context unit
cardinality is refused before invoking it. These are browser admission limits,
not revised native package semantics. Full output is bounded to2MiB and actual
WASM memory to64MiB. Read/startup/computation deadlines and stale-result fences
must preserve accepted output and complete original/current exports.
