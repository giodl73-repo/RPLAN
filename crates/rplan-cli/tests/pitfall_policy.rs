const README: &str = include_str!("../../../README.md");
const COMPATIBILITY: &str = include_str!("../../../docs/compatibility.md");
const U20_CERTIFICATES: &str =
    include_str!("../../../docs/examples/u20-plan-audit-certificates/README.md");
const RPLAN_GOLDEN: &str = include_str!("../../../docs/examples/rplan-golden-packages/README.md");
const RPLAN_BENCHMARK: &str =
    include_str!("../../../docs/examples/rplan-benchmark-packages/README.md");
const RCOUNT_GOLDEN: &str = include_str!("../../../docs/examples/rcount-golden-packages/README.md");
const FOUNDATION_WAVE: &str =
    include_str!("../../../context/waves/2026-05-15-rplan-foundation/WAVE.md");
const CONSUMER_ROLE: &str =
    include_str!("../../../.roles/parliament/consumer-boundary-reviewer.md");
const PITFALLS: &str = include_str!("../../../.pitfall/rplan-pitfalls.md");

fn assert_contains(haystack: &str, needle: &str) {
    let normalized_haystack = haystack.replace("\r\n", "\n");
    let normalized_needle = needle.replace("\r\n", "\n");
    assert!(
        normalized_haystack.contains(&normalized_needle),
        "expected text to contain: {needle}"
    );
}

#[test]
fn certificate_pass_stays_scoped_to_profile_package_verification() {
    assert_contains(PITFALLS, "RPLAN-PF-01");
    assert_contains(PITFALLS, "Certificate Pass Becomes Legal Approval");
    assert_contains(PITFALLS, "audit-certificate auditor");

    assert_contains(U20_CERTIFICATES, "not broad legal judgments");
    assert_contains(
        U20_CERTIFICATES,
        "declared profile, package hashes, context",
    );
    assert_contains(
        RPLAN_GOLDEN,
        "do not prove large-instance quality, legal sufficiency, fairness",
    );
    assert_contains(
        RPLAN_BENCHMARK,
        "does not prove legal sufficiency, fairness, optimality",
    );
    assert_contains(README, "Portable district-plan representation");
}

#[test]
fn pending_consumer_work_does_not_become_adopted_contract() {
    assert_contains(PITFALLS, "RPLAN-PF-02");
    assert_contains(PITFALLS, "Consumer Pending Work Becomes Adopted Contract");
    assert_contains(PITFALLS, "BISECT/RCOUNT");

    assert_contains(
        FOUNDATION_WAVE,
        "| 02 | BISECT dependency rewire | pending |",
    );
    assert_contains(FOUNDATION_WAVE, "| 03 | RCOUNT integration | pending |");
    assert_contains(COMPATIBILITY, "consumers own plan generation");
    assert_contains(
        CONSUMER_ROLE,
        "without letting any one\nconsumer define the package contract",
    );
    assert_contains(
        README,
        "BISECT generation, mapping, reporting, or research workflows",
    );
}

#[test]
fn rcount_rehearsal_remains_release_gate_for_foundation_changes() {
    assert_contains(PITFALLS, "RPLAN-PF-03");
    assert_contains(PITFALLS, "RCOUNT Rehearsal Is Skipped");
    assert_contains(PITFALLS, "RCOUNT integration owner");

    assert_contains(
        COMPATIBILITY,
        "RCOUNT is the required first consumer rehearsal",
    );
    assert_contains(
        COMPATIBILITY,
        "RPLAN foundation changes are not ready until",
    );
    assert_contains(COMPATIBILITY, "the RCOUNT rehearsal pass");
    assert_contains(RCOUNT_GOLDEN, "district-aggregation-rplan");
    assert_contains(RCOUNT_GOLDEN, "stale-plan");
}
