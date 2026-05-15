use std::process::Command;

fn audit_fixture_path(name: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rplan-audit/fixtures")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn profile_fixture_path(name: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../rplan-audit/profiles")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn certificate_fixture_path(name: &str) -> String {
    audit_fixture_path(name)
}

fn public_u20_example_path(name: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/u20-plan-audit-certificates/grid3x3-valid")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn docs_examples_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples")
}

fn run_grid3x3_valid_with_context(context_path: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            context_path,
            "--constraints",
            "plan-shape,population,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap()
}

fn path_context() -> &'static str {
    include_str!("../../rplan-io/src/fixtures/path5.rctx")
}

fn plan(chamber: &str, assignment: &str) -> String {
    format!(
        r#"{{
  "rplan_version": "0.2",
  "plan": {{
    "schema_version": "district-plan-v1",
    "units": {{
      "unit_kind": "tract",
      "state": "WA",
      "year": 2020,
      "canonical_order": "explicit-unit-ids",
      "unit_ids": ["53001000100", "53001000200", "53001000300", "53001000400", "53001000500"],
      "unit_universe_hash": "sha256:path5-unit-universe"
    }},
    "assignment": {assignment},
    "k": 2,
    "display_labels": ["1", "2"],
    "allow_empty_districts": false
  }},
  "metadata": {{
    "label": "wa_path5",
    "jurisdiction": "WA",
    "chamber": "{chamber}",
    "created_at": "2026-05-10T00:00:00Z"
  }},
  "provenance": {{}},
  "geometry": null,
  "extensions": {{}}
}}"#
    )
}

#[test]
fn audit_valid_plan_exits_zero_with_allow_warnings() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    let ctx_path = tmp.path().join("context.rctx");
    std::fs::write(&plan_path, plan("congressional", "[0, 0, 0, 1, 1]")).unwrap();
    std::fs::write(&ctx_path, path_context()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--context",
            ctx_path.to_str().unwrap(),
            "--constraints",
            "plan-shape,contiguity",
            "--allow-warnings",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""result":"pass-with-warnings""#));
}

#[test]
fn audit_disconnected_plan_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    let ctx_path = tmp.path().join("context.rctx");
    std::fs::write(&plan_path, plan("congressional", "[0, 1, 0, 1, 1]")).unwrap();
    std::fs::write(&ctx_path, path_context()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--context",
            ctx_path.to_str().unwrap(),
            "--constraints",
            "contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit failed"));
}

#[test]
fn state_house_without_profile_exits_two() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    std::fs::write(&plan_path, plan("state-house", "[0, 0, 0, 1, 1]")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--constraints",
            "plan-shape",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--legal-profile is required"));
}

#[test]
fn state_house_incomplete_profile_exits_two() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    let profile_path = tmp.path().join("profile.json");
    std::fs::write(&plan_path, plan("state-house", "[0, 0, 0, 1, 1]")).unwrap();
    std::fs::write(
        &profile_path,
        include_str!("../../rplan-audit/profiles/incomplete-state-house-profile.json"),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--legal-profile",
            profile_path.to_str().unwrap(),
            "--constraints",
            "plan-shape",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("state legislative audit requires an explicit legal profile"));
}

#[test]
fn state_house_with_congressional_profile_exits_two() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("plan.rplan");
    std::fs::write(&plan_path, plan("state-house", "[0, 0, 0, 1, 1]")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--legal-profile",
            &profile_fixture_path("us-congressional-project-v1.json"),
            "--constraints",
            "plan-shape",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("legal profile chamber"));
}

#[test]
fn grid3x3_valid_audit_matches_golden_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
            "--constraints",
            "plan-shape,population,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        include_str!("../../rplan-audit/fixtures/grid3x3-valid-certificate.json").trim()
    );
}

#[test]
fn grid3x3_disconnected_audit_matches_golden_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-disconnected.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
            "--constraints",
            "plan-shape,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        include_str!("../../rplan-audit/fixtures/grid3x3-disconnected-certificate.json").trim()
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit failed"));
}

#[test]
fn grid3x3_missing_contiguity_audit_matches_golden_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--constraints",
            "plan-shape,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        include_str!("../../rplan-audit/fixtures/grid3x3-missing-contiguity-certificate.json")
            .trim()
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit failed"));
}

#[test]
fn grid3x3_audit_context_hash_changes_across_contexts() {
    let tmp = tempfile::TempDir::new().unwrap();
    let alt_context_path = tmp.path().join("grid3x3-alt.rctx");
    let mut alt_context =
        rplan_io::read_rctx_str(include_str!("../../rplan-audit/fixtures/grid3x3.rctx")).unwrap();
    alt_context
        .source_hashes
        .entries
        .insert("fixture".to_string(), "sha256:grid3x3-alt".to_string());
    std::fs::write(
        &alt_context_path,
        rplan_io::write_rctx_string(&alt_context).unwrap(),
    )
    .unwrap();

    let original_output = run_grid3x3_valid_with_context(&audit_fixture_path("grid3x3.rctx"));
    let alt_output = run_grid3x3_valid_with_context(alt_context_path.to_str().unwrap());

    assert!(
        original_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&original_output.stderr)
    );
    assert!(
        alt_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&alt_output.stderr)
    );

    let original: serde_json::Value =
        serde_json::from_slice(&original_output.stdout).expect("original certificate JSON");
    let alt: serde_json::Value =
        serde_json::from_slice(&alt_output.stdout).expect("alternate certificate JSON");
    assert_eq!(original["plan_hash"], alt["plan_hash"]);
    assert_ne!(original["context_hash"], alt["context_hash"]);
    assert_ne!(original["content_hash"], alt["content_hash"]);
}

#[test]
fn malformed_plan_exits_two() {
    let tmp = tempfile::TempDir::new().unwrap();
    let plan_path = tmp.path().join("bad.rplan");
    std::fs::write(&plan_path, "{ not-json").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            plan_path.to_str().unwrap(),
            "--constraints",
            "plan-shape",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("parsing plan"));
}

#[test]
fn fixed_generated_at_output_is_stable_across_identical_runs() {
    let first = run_grid3x3_valid_with_context(&audit_fixture_path("grid3x3.rctx"));
    let second = run_grid3x3_valid_with_context(&audit_fixture_path("grid3x3.rctx"));

    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn verify_certificate_accepts_matching_public_fixture_package() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "verify-certificate",
            "--certificate",
            &public_u20_example_path("audit-certificate.json"),
            "--plan",
            &public_u20_example_path("plan.rplan"),
            "--context",
            &public_u20_example_path("context.rctx"),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(stdout["verification"], "pass");
    assert_eq!(stdout["result"], "pass");
}

#[test]
fn verify_certificate_accepts_all_public_golden_packages() {
    let corpus_root = docs_examples_root().join("rplan-golden-packages");
    let mut verified = Vec::new();
    for entry in std::fs::read_dir(&corpus_root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let dir = entry.path();
        let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
            .args([
                "verify-certificate",
                "--certificate",
                dir.join("audit-certificate.json").to_str().unwrap(),
                "--plan",
                dir.join("plan.rplan").to_str().unwrap(),
                "--context",
                dir.join("context.rctx").to_str().unwrap(),
                "--format",
                "json",
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{} stderr: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(stdout["verification"], "pass");
        verified.push(entry.file_name().to_string_lossy().into_owned());
    }
    verified.sort();
    assert_eq!(
        verified,
        vec![
            "T.14+spectral-partitioning",
            "T.15+capacity-constrained-clustering",
            "T.16+hierarchical-regionalization",
            "T.17+flow-construction",
            "U.16+branch-and-cut",
            "U.17+branch-and-price",
            "U.18+local-search-improvement",
            "U.19+selected-frontier",
        ]
    );
}

#[test]
fn verify_certificate_accepts_all_public_method_packages() {
    let corpus_root = docs_examples_root().join("rplan-method-packages");
    let mut verified = Vec::new();
    for entry in std::fs::read_dir(&corpus_root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let dir = entry.path();
        let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
            .args([
                "verify-certificate",
                "--certificate",
                dir.join("audit-certificate.json").to_str().unwrap(),
                "--plan",
                dir.join("plan.rplan").to_str().unwrap(),
                "--context",
                dir.join("context.rctx").to_str().unwrap(),
                "--format",
                "json",
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{} stderr: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(stdout["verification"], "pass");
        verified.push(entry.file_name().to_string_lossy().into_owned());
    }
    verified.sort();
    assert_eq!(
        verified,
        vec![
            "T.14+spectral-generated-synthetic",
            "U.18+local-search-generated-descendant",
        ]
    );
}

#[test]
fn verify_certificate_accepts_all_public_benchmark_packages() {
    let corpus_root = docs_examples_root().join("rplan-benchmark-packages");
    let mut verified = Vec::new();
    for entry in std::fs::read_dir(&corpus_root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let dir = entry.path();
        let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
            .args([
                "verify-certificate",
                "--certificate",
                dir.join("audit-certificate.json").to_str().unwrap(),
                "--plan",
                dir.join("plan.rplan").to_str().unwrap(),
                "--context",
                dir.join("context.rctx").to_str().unwrap(),
                "--format",
                "json",
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{} stderr: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(stdout["verification"], "pass");
        verified.push(entry.file_name().to_string_lossy().into_owned());
    }
    verified.sort();
    assert_eq!(
        verified,
        vec![
            "T.14+spectral-grid10-benchmark",
            "T.15+capacity-path100-benchmark",
            "T.16+regionalization-path100-benchmark",
            "T.17+flow-path100-benchmark",
            "U.16+branch-and-cut-path8-benchmark",
            "U.18+local-search-grid10-benchmark",
            "U.20+audit-grid10-benchmark",
        ]
    );
}

#[test]
fn verify_certificate_rejects_missing_context_for_contextual_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "verify-certificate",
            "--certificate",
            &certificate_fixture_path("grid3x3-valid-certificate.json"),
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires context hash"));
}

#[test]
fn verify_certificate_rejects_tampered_plan_assignment() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "verify-certificate",
            "--certificate",
            &certificate_fixture_path("grid3x3-valid-certificate.json"),
            "--plan",
            &audit_fixture_path("grid3x3-disconnected.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("plan hash mismatch"));
}

#[test]
fn verify_certificate_rejects_tampered_context_hash() {
    let tmp = tempfile::TempDir::new().unwrap();
    let alt_context_path = tmp.path().join("grid3x3-alt.rctx");
    let mut alt_context =
        rplan_io::read_rctx_str(include_str!("../../rplan-audit/fixtures/grid3x3.rctx")).unwrap();
    alt_context
        .source_hashes
        .entries
        .insert("fixture".to_string(), "sha256:grid3x3-reissued".to_string());
    std::fs::write(
        &alt_context_path,
        rplan_io::write_rctx_string(&alt_context).unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "verify-certificate",
            "--certificate",
            &certificate_fixture_path("grid3x3-valid-certificate.json"),
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            alt_context_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("context hash mismatch"));
}

#[test]
fn verify_certificate_rejects_stale_context_with_same_source_hash() {
    let tmp = tempfile::TempDir::new().unwrap();
    let stale_context_path = tmp.path().join("grid3x3-stale.rctx");
    let mut context =
        rplan_io::read_rctx_str(include_str!("../../rplan-audit/fixtures/grid3x3.rctx")).unwrap();
    context.populations.as_mut().unwrap()[0] = 101;
    std::fs::write(
        &stale_context_path,
        rplan_io::write_rctx_string(&context).unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "verify-certificate",
            "--certificate",
            &certificate_fixture_path("grid3x3-valid-certificate.json"),
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            stale_context_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("context hash mismatch"));
}

#[test]
fn verify_certificate_rejects_unit_order_mismatch() {
    let tmp = tempfile::TempDir::new().unwrap();
    let reordered_context_path = tmp.path().join("grid3x3-reordered.rctx");
    let mut context =
        rplan_io::read_rctx_str(include_str!("../../rplan-audit/fixtures/grid3x3.rctx")).unwrap();
    context.units.unit_ids.swap(0, 1);
    std::fs::write(
        &reordered_context_path,
        rplan_io::write_rctx_string(&context).unwrap(),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "verify-certificate",
            "--certificate",
            &certificate_fixture_path("grid3x3-valid-certificate.json"),
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            reordered_context_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("context hash mismatch"));
}

#[test]
fn audit_missing_input_constraint_reports_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
            "--constraints",
            "splits",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(stdout["result"], "fail");
    assert_eq!(stdout["checks"][1]["name"], "splits");
    assert_eq!(stdout["checks"][1]["status"], "missing-input");
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit failed"));
}

#[test]
fn audit_with_lineage_emits_verifiable_certificate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
            "--constraints",
            "plan-shape,population,contiguity",
            "--fixed-generated-at",
            "2026-05-10T00:00:00Z",
            "--lineage-producer-crate",
            "bisect-local-search",
            "--lineage-method",
            "one-move-improvement",
            "--lineage-extra-json",
            r#"{"status":"fixture-solved"}"#,
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cert: rplan_audit::AuditCertificate = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        cert.algorithm_lineage.as_ref().unwrap().producer_crate,
        "bisect-local-search"
    );
    let plan_text = std::fs::read_to_string(audit_fixture_path("grid3x3-valid.rplan")).unwrap();
    let context_text = std::fs::read_to_string(audit_fixture_path("grid3x3.rctx")).unwrap();
    let document = rplan_io::read_rplan_str(&plan_text).unwrap();
    let context = rplan_io::read_rctx_str(&context_text).unwrap();
    rplan_audit::verify_audit_certificate(&cert, Some(&document.plan), Some(&context)).unwrap();
}

#[test]
fn audit_lineage_rejects_reserved_certificate_extra() {
    let output = Command::new(env!("CARGO_BIN_EXE_rplan"))
        .args([
            "audit",
            "--plan",
            &audit_fixture_path("grid3x3-valid.rplan"),
            "--context",
            &audit_fixture_path("grid3x3.rctx"),
            "--lineage-producer-crate",
            "bisect-local-search",
            "--lineage-method",
            "bad-lineage",
            "--lineage-extra-json",
            r#"{"plan_hash":"sha256:attempted-override"}"#,
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("algorithm lineage extra uses reserved certificate field"));
}
