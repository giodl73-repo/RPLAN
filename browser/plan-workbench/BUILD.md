# Build the plan workbench

Use Rust1.95.0 with wasm32, wasm-bindgen-cli0.2.127, Python3.11+, Node22+, Git
and Chromium. Cargo/npm use committed locks. The compatibility recipe fetches
the exact public RCOUNT commit3849b249b6d4f3016d387faef936d82899527141 and uses
explicit local patches to this RPLAN source. No TRACKER checkout is required.

```powershell
rustup toolchain install 1.95.0 --profile minimal
rustup target add wasm32-unknown-unknown --toolchain 1.95.0
cargo +1.95.0 install wasm-bindgen-cli --version 0.2.127 --locked
cargo +1.95.0 fmt --check
cargo +1.95.0 clippy --workspace --all-targets --locked -- -D warnings
cargo +1.95.0 test --workspace --locked
cargo +1.95.0 run -p rplan-cli --locked -- --help
cargo +1.95.0 fmt --manifest-path browser/plan-workbench/Cargo.toml -- --check
cargo +1.95.0 test --manifest-path browser/plan-workbench/Cargo.toml --all-targets --locked
cargo +1.95.0 clippy --manifest-path browser/plan-workbench/Cargo.toml --all-targets --locked -- -D warnings
cargo +1.95.0 clippy --manifest-path browser/plan-workbench/Cargo.toml --lib --target wasm32-unknown-unknown --locked -- -D warnings
python browser/plan-workbench/build-prototype.py --bindgen <path-to-wasm-bindgen>
python browser/plan-workbench/rehearse-consumer.py
cd browser/plan-workbench
npm ci
npx playwright install chromium
npm run test:pages
```

Serve the prefixed site through HTTP; the test runner serves
`target/plan-prototype/RPLAN`. Set CARGO_TARGET_DIR to reuse your cache. Temporary
source and consumer tests use named profiles within that same target, refresh
first-party timestamps and preserve ordinary dev-profile compile-time paths.
The build recipe clears GIT_COMMIT so inherited caller build labels do not
change the independent CLI comparison; exact source identity lives in build.json.
Only owned Git fixtures normalize checkout CRLF to LF; explicit caller-byte cases
retain CRLF/BOM and original opened exports. Native/WASM comparisons use complete
response text, with no JavaScript numeric reserialization. Original CLI audit
comparisons retain all semantic fields except this companion's explicit runtime
label and the resulting identity hashes; produced certificates independently
verify through the original CLI. Certificate verification does not rerun audits.

From a clean committed checkout, run `python browser/plan-workbench/package-site.py`,
then `python browser/plan-workbench/verify-release.py` and
`python browser/plan-workbench/verify-source.py --bindgen <path-to-wasm-bindgen>`.
The last command verifies every source member, rebuilds outside Git in a path
with spaces, compares all native/CLI controls, rehearses pinned RCOUNT and runs
the rebuilt browser suite. It removes inherited RPLAN_URL so it cannot test a
live site instead. Run `cargo +1.95.0 test --workspace --locked` afterward to
confirm ordinary checkout tests still run. On Linux install Chromium's system
dependencies with `npx playwright install --with-deps chromium`.

Budgets are1MiB runtime,2MiB source ZIP and8MiB total static files. The full
native audit/reader/certificate implementation is delivered rather than a
JavaScript policy fork. The UI preserves raw/original/current/accepted and
normalized exports, native outcomes and caller-assertion boundaries in MODEL.md.
