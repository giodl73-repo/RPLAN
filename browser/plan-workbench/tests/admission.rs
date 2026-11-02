use rplan_plan_workbench::evaluate;
use serde_json::{json, Value};

const PLAN: &str = include_str!("../../../crates/rplan-audit/fixtures/grid3x3-valid.rplan");
const DISCONNECTED: &str =
    include_str!("../../../crates/rplan-audit/fixtures/grid3x3-disconnected.rplan");
const CONTEXT: &str = include_str!("../../../crates/rplan-audit/fixtures/grid3x3.rctx");
const PROFILE: &str =
    include_str!("../../../crates/rplan-audit/profiles/us-congressional-project-v1.json");

fn request() -> Value {
    json!({"mode":"audit","plan":PLAN,"context":CONTEXT,"profile":PROFILE,
      "constraints":["plan-shape","population","contiguity"],"generated_at":"2026-05-10T00:00:00Z"})
}
fn run(value: &Value) -> Value {
    serde_json::from_str(&evaluate(&value.to_string())).unwrap()
}

#[test]
fn full_native_audit_edit_and_verification_round_trip() {
    let request = request();
    let report = run(&request);
    assert_eq!(report["kind"], "report");
    assert_eq!(report["certificate"]["result"], "pass");
    assert_eq!(report["certificate"]["checks"].as_array().unwrap().len(), 3);
    assert_eq!(
        report["certificate"]["runtime"]["binary_name"],
        "rplan-plan-workbench"
    );
    let verified = run(
        &json!({"mode":"verify","plan":PLAN,"context":CONTEXT,"certificate":report["certificate"].to_string()}),
    );
    assert_eq!(verified["kind"], "report");
    assert_eq!(
        verified["verification"]["content_hash"],
        report["certificate"]["content_hash"]
    );
    let mut changed = request.clone();
    changed["plan"] = json!(DISCONNECTED);
    let failed = run(&changed);
    assert_eq!(failed["kind"], "report");
    assert_eq!(failed["certificate"]["result"], "fail");
    assert_ne!(report["plan_hash"], failed["plan_hash"]);
}

#[test]
fn native_missing_input_not_evaluated_and_profile_mismatch_stay_visible() {
    let mut input = request();
    input["context"] = Value::Null;
    let report = run(&input);
    assert_eq!(report["kind"], "report");
    assert_eq!(report["certificate"]["result"], "fail");
    assert!(report["certificate"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["status"] == "missing-input"));
    let mut input = request();
    input["constraints"] = json!(["plan-shape", "vra"]);
    let report = run(&input);
    assert_eq!(report["certificate"]["result"], "pass");
    assert!(report["certificate"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["status"] == "not-evaluated"));
    let mut profile: Value = serde_json::from_str(PROFILE).unwrap();
    profile["year"] = json!(2021);
    input["profile"] = json!(profile.to_string());
    let error = run(&input);
    assert_eq!(error["kind"], "error");
    assert!(error["message"].as_str().unwrap().contains("year 2021"));
}

#[test]
fn resource_admission_precedes_native_allocations_and_indexed_arrays() {
    let mut input = request();
    let mut plan: Value = serde_json::from_str(PLAN).unwrap();
    plan["plan"]["k"] = json!(usize::MAX);
    input["plan"] = json!(plan.to_string());
    assert_eq!(run(&input)["kind"], "refused");
    let mut context: rplan_core::RplanContext = serde_json::from_str(CONTEXT).unwrap();
    context.populations = Some(vec![i64::MAX; context.units.unit_ids.len()]);
    let mut input = request();
    input["context"] = json!(rplan_io::write_rctx_string(&context).unwrap());
    assert_eq!(run(&input)["kind"], "refused");
    context.units.unit_ids.truncate(1);
    context.graph = None;
    context.populations = None;
    context.subdivisions = None;
    context.demographics = None;
    context.geometry = None;
    input["context"] = json!(rplan_io::write_rctx_string(&context).unwrap());
    assert_eq!(run(&input)["kind"], "refused");
    input["plan"] = json!(" ".repeat(262_145));
    assert_eq!(run(&input)["kind"], "refused");
    let response: Value = serde_json::from_str(&evaluate(&" ".repeat(1_048_577))).unwrap();
    assert_eq!(response["kind"], "refused");
}

#[test]
fn stale_context_certificate_and_reserved_lineage_use_original_errors() {
    let mut input = request();
    let mut context: Value = serde_json::from_str(CONTEXT).unwrap();
    context["context_hash"] = json!("sha256:stale");
    input["context"] = json!(context.to_string());
    assert!(run(&input)["message"]
        .as_str()
        .unwrap()
        .contains("context_hash mismatch"));
    let report = run(&request());
    let mut certificate = report["certificate"].clone();
    certificate["result"] = json!("fail");
    let error = run(
        &json!({"mode":"verify","plan":PLAN,"context":CONTEXT,"certificate":certificate.to_string()}),
    );
    assert_eq!(error["kind"], "error");
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("content hash mismatch"));
    let mut input = request();
    input["lineage"]=json!(json!({"producer_crate":"untrusted","producer_version":"1","method":"caller","parent_plan_hashes":[],"extra":{"result":"pass"}}).to_string());
    assert!(run(&input)["message"]
        .as_str()
        .unwrap()
        .contains("reserved"));
}

#[test]
fn verifying_rehashed_assertions_does_not_reexecute_the_audit() {
    let report = run(&request());
    let mut certificate: rplan_audit::AuditCertificate =
        serde_json::from_value(report["certificate"].clone()).unwrap();
    certificate.result = rplan_audit::AuditResult::Fail;
    certificate.checks.clear();
    certificate.content_hash = rplan_audit::certificate_content_hash(&certificate).unwrap();
    certificate.certificate_id = certificate.content_hash.clone();
    let verified = run(
        &json!({"mode":"verify","plan":PLAN,"context":CONTEXT,"certificate":serde_json::to_string(&certificate).unwrap()}),
    );
    assert_eq!(verified["kind"], "report");
    assert_eq!(verified["verification"]["result"], "fail");
    assert!(verified["scope"]
        .as_str()
        .unwrap()
        .contains("does not rerun"));
}

#[test]
fn native_known_fields_legacy_and_unknown_input_remain_distinct() {
    let mut input = request();
    let mut plan: Value = serde_json::from_str(PLAN).unwrap();
    plan["unrecognized"] = json!("<script>untrusted💡</script>");
    plan["extensions"]["caller"] = json!({"status":"approved"});
    input["plan"] = json!(plan.to_string());
    let report = run(&input);
    assert_eq!(report["kind"], "report");
    assert!(report["document"].get("unrecognized").is_none());
    assert_eq!(
        report["document"]["extensions"]["caller"]["status"],
        "approved"
    );
    input["unexpected"] = json!(true);
    assert_eq!(run(&input)["kind"], "refused");
    let plan = json!({"rplan_version":"0.1","metadata":{"label":"legacy","state_code":"WA","year":"2020","chamber":"congressional","num_districts":2,"created_at":"2026-05-10"},"assignments":{"53001000100":1,"53001000200":2},"geometry":null});
    let mut input = request();
    input["plan"] = json!(plan.to_string());
    input["context"] = Value::Null;
    input["constraints"] = json!(["plan-shape"]);
    let report = run(&input);
    assert_eq!(report["kind"], "report");
    assert_eq!(report["document"]["rplan_version"], "0.2");
    assert_eq!(report["document"]["plan"]["assignment"], json!([0, 1]));
}

#[test]
fn raw_stdin_limit_is_checked_before_multibyte_decoding() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_rplan-workbench-control"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut bytes = vec![b' '; rplan_plan_workbench::MAX_REQUEST_BYTES];
    bytes.extend_from_slice("💡".as_bytes());
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["kind"], "refused");
    assert_eq!(response["message"], "Request exceeds 1 MiB");
}

#[test]
fn exact_certificate_export_preserves_integers_above_javascript_precision() {
    let mut input = request();
    input["lineage"] = json!(json!({"producer_crate":"caller","producer_version":"1","method":"synthetic","parent_plan_hashes":[],"extra":{"counter":u64::MAX}}).to_string());
    let report = run(&input);
    assert_eq!(report["kind"], "report");
    let raw = report["certificate_json"].as_str().unwrap();
    assert!(raw.contains("18446744073709551615"));
    let verified = run(&json!({"mode":"verify","plan":PLAN,"context":CONTEXT,"certificate":raw}));
    assert_eq!(verified["kind"], "report");
    assert_eq!(
        verified["verification"]["content_hash"],
        report["certificate"]["content_hash"]
    );
}

#[test]
fn claimed_witness_cardinality_is_bounded_before_platform_usize_decoding() {
    let mut certificate: Value = serde_json::from_str(include_str!(
        "../../../crates/rplan-audit/fixtures/grid3x3-disconnected-certificate.json"
    ))
    .unwrap();
    for check in certificate["checks"].as_array_mut().unwrap() {
        for witness in check["witnesses"].as_array_mut().unwrap() {
            if witness["type"] == "contiguity" {
                witness["component_count"] = json!(u64::MAX);
            }
        }
    }
    let response = run(
        &json!({"mode":"verify","certificate":certificate.to_string(),"plan":PLAN,"context":CONTEXT}),
    );
    assert_eq!(response["kind"], "refused");
    assert_eq!(response["message"], "Claimed witness count exceeds 2048");
}
