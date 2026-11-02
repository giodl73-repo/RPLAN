use rplan_audit::{
    audit_plan_with_lineage, parse_plan_chamber, validate_profile_applicability,
    verify_audit_certificate, AlgorithmLineage, AuditCertificate, AuditConstraint, LegalProfile,
    RuntimeProvenance,
};
use serde::Deserialize;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

pub const MAX_REQUEST_BYTES: usize = 1_048_576;
const MAX_DOCUMENT_BYTES: usize = 262_144;
const MAX_POLICY_BYTES: usize = 32_768;
const MAX_REPORT_BYTES: usize = 2_097_152;
const SCOPE: &str = "Native package/audit/hash diagnostics over supplied data. Caller profiles, timestamps, lineage, sources and provenance are unverified. A passing result is not legal approval, fairness, data authenticity, release approval or consumer adoption. Certificate verification checks declared identities; it does not rerun the audit.";

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    Audit,
    Verify,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    mode: Mode,
    plan: Option<String>,
    context: Option<String>,
    profile: Option<String>,
    certificate: Option<String>,
    lineage: Option<String>,
    #[serde(default)]
    constraints: Vec<AuditConstraint>,
    generated_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LineageInput {
    producer_crate: String,
    producer_version: String,
    method: String,
    parent_plan_hashes: Vec<String>,
    extra: Value,
}

fn issue(kind: &str, phase: &str, message: impl ToString) -> Value {
    json!({"kind":kind,"phase":phase,"message":message.to_string(),"scope":SCOPE})
}

pub fn refusal(message: &str) -> String {
    issue("refused", "admission", message).to_string()
}

fn native_error(phase: &str, error: impl ToString) -> Value {
    issue("error", phase, error)
}

fn bounded_json(raw: &str, phase: &str) -> Result<Value, Value> {
    let value: Value = serde_json::from_str(raw).map_err(|e| native_error(phase, e))?;
    let mut stack = vec![&value];
    let mut nodes = 0;
    while let Some(value) = stack.pop() {
        nodes += 1;
        if nodes > 65_536 {
            return Err(issue("refused", phase, "JSON exceeds 65536 nodes"));
        }
        match value {
            Value::String(s) if s.len() > 16_384 => {
                return Err(issue("refused", phase, "Scalar string exceeds 16 KiB"));
            }
            Value::Array(items) => {
                if items.len() > 8192 {
                    return Err(issue("refused", phase, "Array exceeds 8192 entries"));
                }
                stack.extend(items);
            }
            Value::Object(items) => {
                if items.keys().any(|key| key.len() > 16_384) {
                    return Err(issue("refused", phase, "Object key exceeds 16 KiB"));
                }
                stack.extend(items.values());
            }
            _ => {}
        }
    }
    Ok(value)
}

fn document_bounds(value: &Value, context: bool) -> Result<(), Value> {
    let (units, districts) = if context {
        (value.pointer("/units/unit_ids"), None)
    } else if value.get("rplan_version").and_then(Value::as_str) == Some("0.1") {
        let count = value
            .get("assignments")
            .and_then(Value::as_object)
            .map_or(0, |a| a.len());
        if count > 2048 {
            return Err(issue("refused", "plan", "Unit count exceeds 2048"));
        }
        (None, value.pointer("/metadata/num_districts"))
    } else {
        (
            value.pointer("/plan/units/unit_ids"),
            value.pointer("/plan/k"),
        )
    };
    if units
        .and_then(Value::as_array)
        .is_some_and(|a| a.len() > 2048)
        || districts.and_then(Value::as_u64).is_some_and(|k| k > 128)
    {
        return Err(issue(
            "refused",
            "admission",
            "Limit: 2048 units and 128 districts",
        ));
    }
    if context {
        let edges = value
            .pointer("/graph/adjacency")
            .and_then(Value::as_array)
            .map_or(0, |rows| {
                rows.iter()
                    .filter_map(Value::as_array)
                    .map(Vec::len)
                    .sum::<usize>()
            });
        if edges > 8192 {
            return Err(issue("refused", "context", "Graph exceeds 8192 edges"));
        }
        if value
            .get("populations")
            .and_then(Value::as_array)
            .is_some_and(|a| {
                a.iter()
                    .filter_map(Value::as_i64)
                    .any(|p| p.unsigned_abs() > 1_000_000_000_000)
            })
        {
            return Err(issue(
                "refused",
                "context",
                "Population magnitude exceeds 10^12",
            ));
        }
    }
    Ok(())
}

fn report(input: &str) -> Result<Value, Value> {
    let request: Request =
        serde_json::from_str(input).map_err(|e| issue("refused", "request", e))?;
    let fields = [
        (&request.plan, MAX_DOCUMENT_BYTES),
        (&request.context, MAX_DOCUMENT_BYTES),
        (&request.profile, MAX_POLICY_BYTES),
        (&request.certificate, MAX_DOCUMENT_BYTES),
        (&request.lineage, MAX_POLICY_BYTES),
    ];
    if fields
        .iter()
        .any(|(text, limit)| text.as_ref().is_some_and(|s| s.len() > *limit))
        || fields
            .iter()
            .map(|(s, _)| s.as_ref().map_or(0, String::len))
            .sum::<usize>()
            > 786_432
        || request.constraints.len() > 6
        || request.generated_at.as_ref().is_some_and(|s| s.len() > 128)
    {
        return Err(issue(
            "refused",
            "admission",
            "File, aggregate text, constraint or timestamp limit exceeded",
        ));
    }
    if request.mode == Mode::Audit && request.certificate.is_some() {
        return Err(issue(
            "refused",
            "request",
            "Certificate input belongs to verification mode",
        ));
    }
    if request.mode == Mode::Verify
        && (request.profile.is_some()
            || request.lineage.is_some()
            || !request.constraints.is_empty()
            || request.generated_at.is_some())
    {
        return Err(issue(
            "refused",
            "request",
            "Verification takes only certificate and optional plan/context",
        ));
    }
    let document = request
        .plan
        .as_deref()
        .map(|raw| {
            document_bounds(&bounded_json(raw, "plan")?, false)?;
            rplan_io::read_rplan_str(raw).map_err(|e| native_error("plan", e))
        })
        .transpose()?;
    let context = request
        .context
        .as_deref()
        .map(|raw| {
            document_bounds(&bounded_json(raw, "context")?, true)?;
            rplan_io::read_rctx_str(raw).map_err(|e| native_error("context", e))
        })
        .transpose()?;
    let mut response = json!({"kind":"report","mode":if request.mode==Mode::Audit {"audit"} else {"verify"},"scope":SCOPE,
        "document":document,"context":context,
        "normalized_plan":document.as_ref().map(rplan_io::write_rplan_string).transpose().map_err(|e| native_error("plan-export",e))?,
        "normalized_context":context.as_ref().map(rplan_io::write_rctx_string).transpose().map_err(|e| native_error("context-export",e))?,
        "plan_hash":document.as_ref().map(|d|d.plan.plan_hash()).transpose().map_err(|e| native_error("plan-hash",e))?,
        "computed_unit_universe_hash":document.as_ref().map(|d|d.plan.units.compute_unit_universe_hash()).transpose().map_err(|e|native_error("unit-hash",e))?,
        "notes":["Raw originals, current drafts and native normalized/known fields are distinct. Native readers may ignore unknown fields and convert legacy input; keep original exports.",
          "Native plan-shape compares declared unit-universe hashes; recomputed hash display is separate. Native validation does not establish real-world identity or completeness.",
          "Only requested native constraints are evaluated, with plan-shape always included. Not-evaluated checks may coexist with a passing result.",
          "Generated-at and metadata are caller labels. Native certificate content hashes exclude generated-at, certificate-id and content-hash; identity checks are not signatures or audit re-execution."]});
    if request.mode == Mode::Audit {
        let document = document
            .as_ref()
            .ok_or_else(|| issue("refused", "request", "Audit requires a plan"))?;
        if context
            .as_ref()
            .is_some_and(|c| c.units.unit_ids.len() != document.plan.units.unit_ids.len())
        {
            return Err(issue(
                "refused",
                "admission",
                "Native audit assumes equal plan/context indexed-array cardinalities",
            ));
        }
        let raw = request
            .profile
            .as_deref()
            .ok_or_else(|| issue("refused", "request", "Audit requires an explicit profile"))?;
        bounded_json(raw, "profile")?;
        let profile: LegalProfile =
            serde_json::from_str(raw).map_err(|e| native_error("profile", e))?;
        let chamber = parse_plan_chamber(&document.metadata.chamber)
            .map_err(|e| native_error("profile", e))?;
        validate_profile_applicability(document, &chamber, &profile)
            .map_err(|e| native_error("profile", e))?;
        let lineage = request
            .lineage
            .as_deref()
            .map(|raw| {
                bounded_json(raw, "lineage")?;
                let input: LineageInput =
                    serde_json::from_str(raw).map_err(|e| native_error("lineage", e))?;
                AlgorithmLineage::new(
                    input.producer_crate,
                    input.producer_version,
                    input.method,
                    input.parent_plan_hashes,
                    input.extra,
                )
                .map_err(|e| native_error("lineage", e))
            })
            .transpose()?;
        let generated = request.generated_at.as_deref().ok_or_else(|| {
            issue(
                "refused",
                "request",
                "Audit requires a caller generated-at label",
            )
        })?;
        let runtime = RuntimeProvenance {
            binary_name: "rplan-plan-workbench".into(),
            binary_version: env!("CARGO_PKG_VERSION").into(),
            ..Default::default()
        };
        let certificate = audit_plan_with_lineage(
            &document.plan,
            context.as_ref(),
            &profile,
            runtime,
            &request.constraints,
            generated,
            lineage,
        )
        .map_err(|e| native_error("native-audit", e))?;
        response["certificate_json"] = json!(serde_json::to_string_pretty(&certificate)
            .map_err(|e| native_error("certificate-export", e))?);
        response["certificate"] = json!(certificate);
        response["profile"] = json!(profile);
        response["requested_constraints"] = json!(request.constraints);
    } else {
        let raw = request
            .certificate
            .as_deref()
            .ok_or_else(|| issue("refused", "request", "Verification requires a certificate"))?;
        let value = bounded_json(raw, "certificate")?;
        if let Some(checks) = value.get("checks").and_then(Value::as_array) {
            for witness in checks
                .iter()
                .filter_map(|c| c.get("witnesses").and_then(Value::as_array))
                .flatten()
            {
                let count = match witness.get("type").and_then(Value::as_str) {
                    Some("contiguity") => witness.get("component_count"),
                    Some("split") => witness.get("unit_count"),
                    Some("geometry") => witness.get("unit_geometry_hash_count"),
                    _ => None,
                };
                if count.and_then(Value::as_u64).is_some_and(|n| n > 2048) {
                    return Err(issue(
                        "refused",
                        "certificate",
                        "Claimed witness count exceeds 2048",
                    ));
                }
            }
        }
        let certificate: AuditCertificate =
            serde_json::from_str(raw).map_err(|e| native_error("certificate", e))?;
        let verified = verify_audit_certificate(
            &certificate,
            document.as_ref().map(|d| &d.plan),
            context.as_ref(),
        )
        .map_err(|e| native_error("native-verification", e))?;
        response["verification"] = json!({"certificate_id":verified.certificate_id,"content_hash":verified.content_hash,"plan_hash":verified.plan_hash,"context_hash":verified.context_hash,"result":verified.result});
        response["certificate_json"] = json!(serde_json::to_string_pretty(&certificate)
            .map_err(|e| native_error("certificate-export", e))?);
        response["certificate"] = json!(certificate);
    }
    Ok(response)
}

#[wasm_bindgen]
pub fn engine_info() -> String {
    json!({"constraints":[AuditConstraint::PlanShape,AuditConstraint::Population,AuditConstraint::Contiguity,AuditConstraint::Splits,AuditConstraint::Vra,AuditConstraint::Geometry],
        "file_limits":{"plan":MAX_DOCUMENT_BYTES,"context":MAX_DOCUMENT_BYTES,"profile":MAX_POLICY_BYTES,"certificate":MAX_DOCUMENT_BYTES,"lineage":MAX_POLICY_BYTES},
        "request_bytes":MAX_REQUEST_BYTES,"report_bytes":MAX_REPORT_BYTES,"scope":SCOPE}).to_string()
}

#[wasm_bindgen]
pub fn evaluate(input: &str) -> String {
    if input.len() > MAX_REQUEST_BYTES {
        return refusal("Request exceeds 1 MiB");
    }
    let output = report(input).unwrap_or_else(|error| error).to_string();
    if output.len() > MAX_REPORT_BYTES {
        refusal("Whole report exceeds 2 MiB")
    } else {
        output
    }
}
