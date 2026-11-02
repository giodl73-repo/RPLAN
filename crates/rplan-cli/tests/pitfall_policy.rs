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
const AUTHORITY_BOUNDARIES: &str = include_str!("../../../docs/authority-boundaries.v1.json");

fn assert_contains(haystack: &str, needle: &str) {
    let normalized_haystack = haystack.replace("\r\n", "\n");
    let normalized_needle = needle.replace("\r\n", "\n");
    assert!(
        normalized_haystack.contains(&normalized_needle),
        "expected text to contain: {needle}"
    );
}

fn load_boundaries() -> serde_json::Value {
    let boundaries: serde_json::Value =
        serde_json::from_str(AUTHORITY_BOUNDARIES).expect("authority boundaries parse");
    assert_eq!(boundaries["schema"], "rplan.authority-boundaries.v1");
    assert_eq!(boundaries["repo"], "RPLAN");
    boundaries
}

fn pitfall<'a>(boundaries: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    let pitfall = boundaries["pitfalls"]
        .as_array()
        .expect("pitfalls is an array")
        .iter()
        .find(|pitfall| pitfall["id"] == id)
        .unwrap_or_else(|| panic!("{id} boundary is recorded"));
    assert_eq!(pitfall["status"], "mitigated");
    pitfall
}

fn assert_array_contains(pitfall: &serde_json::Value, field: &str, expected: &[&str]) {
    let actual = pitfall[field]
        .as_array()
        .unwrap_or_else(|| panic!("{field} must be an array"));
    for value in expected {
        assert!(
            actual.iter().any(|actual| actual == value),
            "{field} must contain {value}"
        );
    }
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

    let boundaries = load_boundaries();
    let certificate = pitfall(&boundaries, "RPLAN-PF-01");
    assert_contains(
        certificate["boundary"].as_str().expect("boundary is text"),
        "verifies only the declared profile",
    );
    assert_array_contains(
        certificate,
        "not_authorized",
        &[
            "legal_approval",
            "fairness_proof",
            "public_release_approval",
            "policy_preference",
            "large_instance_quality_claim",
            "full_redistricting_acceptance",
        ],
    );
    assert_array_contains(
        certificate,
        "required_before_upgrade",
        &[
            "downstream_publication_owner",
            "legal_review",
            "fairness_review",
            "real_data_quality_review",
            "release_review",
        ],
    );
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

    let boundaries = load_boundaries();
    let consumer = pitfall(&boundaries, "RPLAN-PF-02");
    assert_contains(
        consumer["boundary"].as_str().expect("boundary is text"),
        "does not mean BISECT, RCOUNT, CROP",
    );
    assert_array_contains(
        consumer,
        "not_authorized",
        &[
            "bisect_dependency_rewire_complete",
            "rcount_integration_complete",
            "crop_adoption_complete",
            "consumer_contract_adopted",
            "customer_ready_dependency_status",
        ],
    );
    assert_array_contains(
        consumer,
        "required_before_upgrade",
        &[
            "consumer_owned_change",
            "consumer_validation_pass",
            "migration_notes",
            "consumer_boundary_review",
            "tracker_pointer_snapshot",
        ],
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

    let boundaries = load_boundaries();
    let rehearsal = pitfall(&boundaries, "RPLAN-PF-03");
    assert_contains(
        rehearsal["boundary"].as_str().expect("boundary is text"),
        "not release-ready after RPLAN-local tests alone",
    );
    assert_array_contains(
        rehearsal,
        "not_authorized",
        &[
            "rplan_local_tests_only_release",
            "hash_contract_ready_without_rcount",
            "package_format_ready_without_rcount",
            "audit_semantics_ready_without_rcount",
            "portfolio_pointer_promotion_without_rehearsal",
        ],
    );
    assert_array_contains(
        rehearsal,
        "required_before_upgrade",
        &[
            "affected_foundation_tests",
            "rcount_rehearsal_pass",
            "or_explicit_rehearsal_scope_out",
            "consumer_boundary_review",
            "compatibility_policy_update",
        ],
    );
}
