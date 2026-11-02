# RPLAN browser plan audit: target contract

A reader opens or edits their own RPLAN0.2/retained0.1 plan and optional RCTX
context, supplies an explicit profile, chooses native constraints, runs the
original Rust audit and inspects every check, warning, witness and certificate.
They can also verify a supplied certificate against the current plan/context.
Synthetic positive and disconnected examples work immediately. Original bytes,
current drafts, accepted input, native report, normalized plan/context and audit
certificate downloads are separate. This is a dedicated reader product.

REQ-BROWSER-001 / SPEC-BROWSER-001 / WP-BROWSER-001 are targets until their gates
pass. No native foundation, real-world legal/fairness, provenance authentication,
consumer adoption or publication gate is closed by a browser report.

## Native ownership and compatibility

Call existing rplan-io readers/writers, core hash/validation and audit/certificate
functions. Extract existing private CLI chamber/profile applicability checks
into portable audit helpers and delegate CLI calls; preserve error text/order,
root manifests/lock, every original test and all original policy/hash behavior.
The browser requires an explicit profile rather than inventing applicable law.
All six native constraints are available; native not-evaluated, missing-input,
fail and pass-with-warnings states remain distinct. Runtime labels identify this
companion; generated-at is a caller label, not an attested wall clock. Optional
lineage uses native AlgorithmLineage construction and reserved-field checks.
Certificate verification checks native declared content/plan/context identities;
it does not rerun the requested audit or establish evidence authenticity.

Preserve native quirks explicitly: declared unit-universe hashes are not generally
validated against recomputation; some schema/profile fields are caller strings;
not-evaluated constraints may coexist with a passing overall result. Identity
hashes, raw/known fields and unverified provenance are separate. Unknown input
fields and native conversions must be disclosed and complete raw input retained.

## Admission and custody

Use a separate locked Rust/WASM companion and real module worker. Bound the raw
request before parsing (1MiB), each plan/context (256KiB), profile/lineage (32KiB),
certificate (256KiB), total admitted text (768KiB), JSON depth (native serde bound),
units/known claimed witness counts (2048), districts (128), edges (8192), generic arrays (8192), scalar strings
(16KiB), whole report (2MiB) and actual compiled memory (64MiB). Population absolute
values at most10^12 keep native i64 sums/differences safe within those unit bounds.
Reject unsafe plan/context array cardinality combinations before native auditing,
which assumes compatible indexed arrays. This is browser admission, not a changed
native validator. Refuse oversized operations whole without truncation.

Use real5s read,12s startup and10s computation deadlines plus generation/job/input
revision fences. Failures/retry preserve accepted results and original/current
exports. No supplied text, URLs, scripts, commands or provenance are executed or
fetched. Render all user-controlled strings as text. Support keyboard and390px
mobile use. Source/build/license/compiled-download routes work after engine failure.

## Qualification and delivery gates

1. Baseline70 original native tests, strict lint/fmt and wasm32 compilation.
   Capture independent original CLI positive/negative/profile/certificate outputs
   before helper extraction; preserve bytes afterward, and guard original tests.
2. Complete adapter native/WASM outcome parity, independent original CLI semantic
   audit/certificate controls, boundary/unsafe-array/unknown/malformed/Unicode cases,
   actual memory maximum, native-to-browser certificate replay and own edits.
3. RCOUNT's three required district aggregation compatibility rehearsals use this
   RPLAN source through explicit local patches in a separate owned RCOUNT worktree.
   They prove the tested compatibility, not BISECT/RCOUNT deployment or adoption.
4. Browser worker/task/recovery/mobile/keyboard tests, exact source/legal/runtime
   packaging, independent outside-Git source rebuild, qualified PR and default
   deployment, fresh live tests and every served hash. Use the same shared Cargo
   target, a separate temporary-source profile and ordinary native tests afterward.
5. Push child source first. Advance canonical RPLAN and TRACKER only in an explicit
   qualified portfolio snapshot. Preserve original531a8a9 PITFALL history remotely.
   RCOUNT canonical and all BISECT/ICELINES/TIG/REEL owner work stay outside this slice.
