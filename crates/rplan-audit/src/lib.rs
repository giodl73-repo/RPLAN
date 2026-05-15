use rplan_core::{canonical_sha256, DistrictPlan, RplanContext, SourceHashes, SubdivisionContext};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use thiserror::Error;

pub const AUDIT_CERTIFICATE_SCHEMA_VERSION: &str = "audit-certificate-v1";
pub const LEGAL_PROFILE_SCHEMA_VERSION: &str = "legal-profile-v1";

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("core error: {0}")]
    Core(#[from] rplan_core::RplanCoreError),
    #[error("certificate content hash error: {0}")]
    Hash(String),
    #[error("certificate schema version mismatch: {0}")]
    CertificateSchema(String),
    #[error("certificate id mismatch: expected {expected}, found {found}")]
    CertificateIdMismatch { expected: String, found: String },
    #[error("certificate content hash mismatch: expected {expected}, found {found}")]
    CertificateContentHashMismatch { expected: String, found: String },
    #[error("certificate plan hash mismatch: expected {expected}, found {found}")]
    CertificatePlanHashMismatch { expected: String, found: String },
    #[error("certificate plan schema mismatch: expected {expected}, found {found}")]
    CertificatePlanSchemaMismatch { expected: String, found: String },
    #[error("certificate context hash mismatch: expected {expected}, found {found}")]
    CertificateContextHashMismatch { expected: String, found: String },
    #[error("certificate requires context hash {0}, but no context was supplied")]
    CertificateContextMissing(String),
    #[error("certificate source hashes do not match context source hashes")]
    CertificateSourceHashesMismatch,
    #[error("algorithm lineage extra uses reserved certificate field: {0}")]
    AlgorithmLineageExtraReservedField(String),
    #[error("state legislative audit requires an explicit legal profile")]
    MissingStateLegislativeProfile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Chamber {
    Congressional,
    StateHouse,
    StateSenate,
    Local,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum PopulationToleranceRule {
    ExactAbsolute { max_total_deviation: i64 },
    ExactPpm { max_deviation_ppm: i64 },
    Percent { max_deviation_percent: f64 },
    StateSpecific { rule_id: String },
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum SplitRule {
    NotEvaluated,
    CountOnly,
    MinimizeWherePracticable,
    PreserveUnlessNecessary,
    StateSpecific { rule_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum NestingRule {
    NotEvaluated,
    StateSpecific { rule_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum VraPolicy {
    NotEvaluated,
    ReportOpportunityDistricts {
        minority_group: String,
        vap_threshold: f64,
    },
    StateSpecific {
        rule_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegalProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub jurisdiction: String,
    pub chamber: Chamber,
    pub year: u16,
    pub population_tolerance: PopulationToleranceRule,
    pub contiguity_required: bool,
    pub county_split_rule: SplitRule,
    pub municipal_split_rule: SplitRule,
    pub nesting_rule: NestingRule,
    pub vra_policy: VraPolicy,
}

impl LegalProfile {
    pub fn us_congressional_project_v1(year: u16) -> Self {
        Self {
            schema_version: LEGAL_PROFILE_SCHEMA_VERSION.to_string(),
            profile_id: "US_CONGRESSIONAL_PROJECT_V1".to_string(),
            jurisdiction: "US".to_string(),
            chamber: Chamber::Congressional,
            year,
            population_tolerance: PopulationToleranceRule::ExactPpm {
                max_deviation_ppm: 5000,
            },
            contiguity_required: true,
            county_split_rule: SplitRule::CountOnly,
            municipal_split_rule: SplitRule::NotEvaluated,
            nesting_rule: NestingRule::NotEvaluated,
            vra_policy: VraPolicy::NotEvaluated,
        }
    }

    pub fn hash(&self) -> Result<String, AuditError> {
        let value = serde_json::to_value(self).map_err(|err| AuditError::Hash(err.to_string()))?;
        canonical_sha256(&value).map_err(AuditError::from)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RuntimeProvenance {
    pub binary_name: String,
    pub binary_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solver: Option<SolverProvenance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolverProvenance {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_limit_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimality_gap: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlgorithmLineage {
    pub producer_crate: String,
    pub producer_version: String,
    pub method: String,
    pub parent_plan_hashes: Vec<String>,
    pub parameters_hash: String,
    pub extra: serde_json::Value,
}

impl AlgorithmLineage {
    pub fn new(
        producer_crate: impl Into<String>,
        producer_version: impl Into<String>,
        method: impl Into<String>,
        parent_plan_hashes: Vec<String>,
        extra: serde_json::Value,
    ) -> Result<Self, AuditError> {
        let lineage = Self {
            producer_crate: producer_crate.into(),
            producer_version: producer_version.into(),
            method: method.into(),
            parent_plan_hashes,
            parameters_hash: canonical_sha256(&extra).map_err(AuditError::from)?,
            extra,
        };
        validate_algorithm_lineage_extra(&lineage)?;
        Ok(lineage)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditResult {
    Pass,
    Fail,
    PassWithWarnings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    Pass,
    Fail,
    NotEvaluated,
    MissingInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditConstraint {
    PlanShape,
    Population,
    Contiguity,
    Splits,
    Vra,
    Geometry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditWarning {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_check: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditCheck {
    pub name: String,
    pub status: CheckStatus,
    pub severity: Severity,
    pub summary: String,
    pub witnesses: Vec<Witness>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Witness {
    Population(PopulationWitness),
    Contiguity(ContiguityWitness),
    MissingInput(MissingInputWitness),
    Split(SplitWitness),
    Vra(VraWitness),
    Geometry(GeometryWitness),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationWitness {
    pub district_id: u32,
    pub population: i64,
    pub ideal: f64,
    pub absolute_deviation: f64,
    pub percent_deviation: f64,
    pub deviation_ppm: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContiguityWitness {
    pub district_id: u32,
    pub component_count: usize,
    pub component_unit_ids: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingInputWitness {
    pub input: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitWitness {
    pub subdivision_kind: String,
    pub subdivision_id: String,
    pub district_ids: Vec<u32>,
    pub unit_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VraWitness {
    pub district_id: u32,
    pub minority_group: String,
    pub total_vap: f64,
    pub minority_vap: f64,
    pub minority_vap_percent: f64,
    pub threshold_percent: f64,
    pub is_opportunity_district: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GeometryWitness {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<String>,
    pub unit_geometry_hash_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegalProfileSummary {
    pub schema_version: String,
    pub profile_id: String,
    pub jurisdiction: String,
    pub chamber: Chamber,
    pub year: u16,
    pub population_tolerance: PopulationToleranceRule,
    pub county_split_rule: SplitRule,
    pub municipal_split_rule: SplitRule,
    pub vra_policy: VraPolicy,
    pub legal_disclaimer: String,
}

impl From<&LegalProfile> for LegalProfileSummary {
    fn from(profile: &LegalProfile) -> Self {
        Self {
            schema_version: profile.schema_version.clone(),
            profile_id: profile.profile_id.clone(),
            jurisdiction: profile.jurisdiction.clone(),
            chamber: profile.chamber.clone(),
            year: profile.year,
            population_tolerance: profile.population_tolerance.clone(),
            county_split_rule: profile.county_split_rule.clone(),
            municipal_split_rule: profile.municipal_split_rule.clone(),
            vra_policy: profile.vra_policy.clone(),
            legal_disclaimer:
                "Pass means the plan passes the supplied profile, not every possible legal requirement."
                    .to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditCertificate {
    pub schema_version: String,
    pub certificate_id: String,
    pub generated_at_utc: String,
    pub content_hash: String,
    pub plan_hash: String,
    pub plan_schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_hash: Option<String>,
    pub legal_profile_hash: String,
    pub legal_profile: LegalProfileSummary,
    pub source_hashes: SourceHashes,
    pub runtime: RuntimeProvenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm_lineage: Option<AlgorithmLineage>,
    pub result: AuditResult,
    pub checks: Vec<AuditCheck>,
    pub warnings: Vec<AuditWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditCertificateVerification {
    pub certificate_id: String,
    pub content_hash: String,
    pub plan_hash: String,
    pub context_hash: Option<String>,
    pub result: AuditResult,
}

pub fn audit_plan(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
    profile: &LegalProfile,
    runtime: RuntimeProvenance,
    constraints: &[AuditConstraint],
    generated_at_utc: &str,
) -> Result<AuditCertificate, AuditError> {
    audit_plan_with_lineage(
        plan,
        context,
        profile,
        runtime,
        constraints,
        generated_at_utc,
        None,
    )
}

pub fn audit_plan_with_lineage(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
    profile: &LegalProfile,
    runtime: RuntimeProvenance,
    constraints: &[AuditConstraint],
    generated_at_utc: &str,
    algorithm_lineage: Option<AlgorithmLineage>,
) -> Result<AuditCertificate, AuditError> {
    if matches!(
        profile.chamber,
        Chamber::StateHouse | Chamber::StateSenate | Chamber::Local
    ) && profile.profile_id.is_empty()
    {
        return Err(AuditError::MissingStateLegislativeProfile);
    }
    if let Some(lineage) = &algorithm_lineage {
        validate_algorithm_lineage_extra(lineage)?;
    }

    let requested: BTreeSet<AuditConstraint> = constraints.iter().cloned().collect();
    let mut checks = vec![check_plan_shape(plan, context)];
    if requested.contains(&AuditConstraint::Population) {
        checks.push(check_population(plan, context, profile));
    }
    if requested.contains(&AuditConstraint::Contiguity) {
        checks.push(check_contiguity(plan, context, profile));
    }
    if requested.contains(&AuditConstraint::Splits) {
        checks.push(check_splits(plan, context, profile));
    }
    if requested.contains(&AuditConstraint::Vra) {
        checks.push(check_vra(plan, context, profile));
    }
    if requested.contains(&AuditConstraint::Geometry) {
        checks.push(check_geometry(plan, context));
    }

    let warnings = provenance_warnings(context);
    let result = result_for(&checks, &warnings);
    let plan_hash = plan.plan_hash()?;
    let legal_profile_hash = profile.hash()?;
    let source_hashes = context
        .map(|ctx| ctx.source_hashes.clone())
        .unwrap_or_default();

    let mut certificate = AuditCertificate {
        schema_version: AUDIT_CERTIFICATE_SCHEMA_VERSION.to_string(),
        certificate_id: String::new(),
        generated_at_utc: generated_at_utc.to_string(),
        content_hash: String::new(),
        plan_hash,
        plan_schema_version: plan.schema_version.clone(),
        context_hash: context.map(|ctx| ctx.context_hash.clone()),
        legal_profile_hash,
        legal_profile: LegalProfileSummary::from(profile),
        source_hashes,
        runtime,
        algorithm_lineage,
        result,
        checks,
        warnings,
    };
    certificate.content_hash = certificate_content_hash(&certificate)?;
    certificate.certificate_id = certificate.content_hash.clone();
    Ok(certificate)
}

pub fn certificate_content_hash(certificate: &AuditCertificate) -> Result<String, AuditError> {
    let mut value =
        serde_json::to_value(certificate).map_err(|err| AuditError::Hash(err.to_string()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| AuditError::Hash("certificate is not a JSON object".to_string()))?;
    object.remove("certificate_id");
    object.remove("generated_at_utc");
    object.remove("content_hash");
    canonical_sha256(&value).map_err(AuditError::from)
}

pub fn verify_audit_certificate(
    certificate: &AuditCertificate,
    plan: Option<&DistrictPlan>,
    context: Option<&RplanContext>,
) -> Result<AuditCertificateVerification, AuditError> {
    if certificate.schema_version != AUDIT_CERTIFICATE_SCHEMA_VERSION {
        return Err(AuditError::CertificateSchema(
            certificate.schema_version.clone(),
        ));
    }
    if let Some(lineage) = &certificate.algorithm_lineage {
        validate_algorithm_lineage_extra(lineage)?;
    }

    let computed_content_hash = certificate_content_hash(certificate)?;
    if certificate.content_hash != computed_content_hash {
        return Err(AuditError::CertificateContentHashMismatch {
            expected: computed_content_hash,
            found: certificate.content_hash.clone(),
        });
    }
    if certificate.certificate_id != certificate.content_hash {
        return Err(AuditError::CertificateIdMismatch {
            expected: certificate.content_hash.clone(),
            found: certificate.certificate_id.clone(),
        });
    }

    if let Some(plan) = plan {
        plan.validate()?;
        let plan_hash = plan.plan_hash()?;
        if certificate.plan_hash != plan_hash {
            return Err(AuditError::CertificatePlanHashMismatch {
                expected: certificate.plan_hash.clone(),
                found: plan_hash,
            });
        }
        if certificate.plan_schema_version != plan.schema_version {
            return Err(AuditError::CertificatePlanSchemaMismatch {
                expected: certificate.plan_schema_version.clone(),
                found: plan.schema_version.clone(),
            });
        }
    }

    if let Some(context) = context {
        context.validate()?;
        let computed_context_hash = context.compute_context_hash()?;
        if context.context_hash != computed_context_hash {
            return Err(AuditError::CertificateContextHashMismatch {
                expected: computed_context_hash,
                found: context.context_hash.clone(),
            });
        }
        if certificate.context_hash.as_deref() != Some(computed_context_hash.as_str()) {
            return Err(AuditError::CertificateContextHashMismatch {
                expected: certificate.context_hash.clone().unwrap_or_default(),
                found: computed_context_hash,
            });
        }
        if certificate.source_hashes != context.source_hashes {
            return Err(AuditError::CertificateSourceHashesMismatch);
        }
    } else if let Some(context_hash) = &certificate.context_hash {
        return Err(AuditError::CertificateContextMissing(context_hash.clone()));
    }

    Ok(AuditCertificateVerification {
        certificate_id: certificate.certificate_id.clone(),
        content_hash: certificate.content_hash.clone(),
        plan_hash: certificate.plan_hash.clone(),
        context_hash: certificate.context_hash.clone(),
        result: certificate.result.clone(),
    })
}

fn validate_algorithm_lineage_extra(lineage: &AlgorithmLineage) -> Result<(), AuditError> {
    let Some(extra) = lineage.extra.as_object() else {
        return Ok(());
    };
    const RESERVED_CERTIFICATE_FIELDS: &[&str] = &[
        "schema_version",
        "certificate_id",
        "generated_at_utc",
        "content_hash",
        "plan_hash",
        "plan_schema_version",
        "context_hash",
        "legal_profile_hash",
        "legal_profile",
        "source_hashes",
        "runtime",
        "algorithm_lineage",
        "result",
        "checks",
        "warnings",
    ];
    for field in RESERVED_CERTIFICATE_FIELDS {
        if extra.contains_key(*field) {
            return Err(AuditError::AlgorithmLineageExtraReservedField(
                (*field).to_string(),
            ));
        }
    }
    Ok(())
}

fn check_plan_shape(plan: &DistrictPlan, context: Option<&RplanContext>) -> AuditCheck {
    let mut witnesses = Vec::new();
    let mut failures = Vec::new();
    if let Err(err) = plan.validate() {
        failures.push(err.to_string());
    }
    if let Some(context) = context {
        if plan.units.unit_universe_hash != context.units.unit_universe_hash {
            failures.push("PLAN_CONTEXT_UNIT_UNIVERSE_MISMATCH".to_string());
        }
    }

    if failures.is_empty() {
        AuditCheck {
            name: "plan-shape".to_string(),
            status: CheckStatus::Pass,
            severity: Severity::Info,
            summary: "plan shape is valid".to_string(),
            witnesses,
        }
    } else {
        witnesses.push(Witness::MissingInput(MissingInputWitness {
            input: "plan-shape".to_string(),
            reason: failures.join("; "),
        }));
        AuditCheck {
            name: "plan-shape".to_string(),
            status: CheckStatus::Fail,
            severity: Severity::Error,
            summary: failures.join("; "),
            witnesses,
        }
    }
}

fn check_population(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
    profile: &LegalProfile,
) -> AuditCheck {
    let Some(populations) = context.and_then(|ctx| ctx.populations.as_ref()) else {
        return missing_input_check(
            "population",
            "populations",
            "population audit requires context populations",
        );
    };
    let total: i64 = populations.iter().sum();
    let ideal = total as f64 / plan.k as f64;
    let mut district_populations = vec![0i64; plan.k];
    for (idx, &district_id) in plan.assignment.iter().enumerate() {
        if let Some(slot) = district_populations.get_mut(district_id as usize) {
            *slot += populations[idx];
        }
    }

    let mut failing = Vec::new();
    let mut witnesses = Vec::new();
    let max_population = district_populations.iter().copied().max().unwrap_or(0);
    let min_population = district_populations.iter().copied().min().unwrap_or(0);
    let total_deviation = max_population - min_population;
    for (district_id, population) in district_populations.into_iter().enumerate() {
        let absolute_deviation = (population as f64 - ideal).abs();
        let percent_deviation = if ideal == 0.0 {
            0.0
        } else {
            absolute_deviation / ideal * 100.0
        };
        let deviation_ppm = if ideal == 0.0 {
            0
        } else {
            (absolute_deviation / ideal * 1_000_000.0).round() as i64
        };
        let witness = PopulationWitness {
            district_id: district_id as u32,
            population,
            ideal,
            absolute_deviation,
            percent_deviation,
            deviation_ppm,
        };
        if !population_passes(&witness, &profile.population_tolerance, total_deviation) {
            failing.push(district_id as u32);
            witnesses.push(Witness::Population(witness));
        }
    }

    if matches!(
        profile.population_tolerance,
        PopulationToleranceRule::Unspecified
    ) {
        return AuditCheck {
            name: "population".to_string(),
            status: CheckStatus::MissingInput,
            severity: Severity::Error,
            summary: "population profile is unspecified".to_string(),
            witnesses: vec![Witness::MissingInput(MissingInputWitness {
                input: "legal_profile.population_tolerance".to_string(),
                reason: "population audit requires a tolerance rule".to_string(),
            })],
        };
    }

    if failing.is_empty() {
        AuditCheck {
            name: "population".to_string(),
            status: CheckStatus::Pass,
            severity: Severity::Info,
            summary: "all districts pass population tolerance".to_string(),
            witnesses,
        }
    } else {
        AuditCheck {
            name: "population".to_string(),
            status: CheckStatus::Fail,
            severity: Severity::Error,
            summary: format!("districts {:?} exceed population tolerance", failing),
            witnesses,
        }
    }
}

fn population_passes(
    witness: &PopulationWitness,
    rule: &PopulationToleranceRule,
    total_deviation: i64,
) -> bool {
    match rule {
        PopulationToleranceRule::ExactAbsolute {
            max_total_deviation,
        } => total_deviation <= *max_total_deviation,
        PopulationToleranceRule::ExactPpm { max_deviation_ppm } => {
            witness.deviation_ppm <= *max_deviation_ppm
        }
        PopulationToleranceRule::Percent {
            max_deviation_percent,
        } => witness.percent_deviation <= *max_deviation_percent,
        PopulationToleranceRule::StateSpecific { .. } | PopulationToleranceRule::Unspecified => {
            false
        }
    }
}

fn check_contiguity(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
    profile: &LegalProfile,
) -> AuditCheck {
    if !profile.contiguity_required {
        return AuditCheck {
            name: "contiguity".to_string(),
            status: CheckStatus::NotEvaluated,
            severity: Severity::Info,
            summary: "contiguity is not required by the supplied profile".to_string(),
            witnesses: Vec::new(),
        };
    }
    let Some(graph) = context.and_then(|ctx| ctx.graph.as_ref()) else {
        return missing_input_check(
            "contiguity",
            "graph",
            "contiguity audit requires context graph",
        );
    };
    let context = context.expect("context exists when graph exists");
    let mut witnesses = Vec::new();
    for district_id in 0..plan.k {
        let members: Vec<usize> = plan
            .assignment
            .iter()
            .enumerate()
            .filter_map(|(idx, &assigned)| (assigned as usize == district_id).then_some(idx))
            .collect();
        let components = district_components(&members, &graph.adjacency);
        if components.len() > 1 {
            let component_count = components.len();
            let component_unit_ids = components
                .into_iter()
                .map(|component| {
                    component
                        .into_iter()
                        .map(|idx| context.units.unit_ids[idx].clone())
                        .collect()
                })
                .collect();
            witnesses.push(Witness::Contiguity(ContiguityWitness {
                district_id: district_id as u32,
                component_count,
                component_unit_ids,
            }));
        }
    }

    if witnesses.is_empty() {
        AuditCheck {
            name: "contiguity".to_string(),
            status: CheckStatus::Pass,
            severity: Severity::Info,
            summary: "all districts are contiguous".to_string(),
            witnesses,
        }
    } else {
        AuditCheck {
            name: "contiguity".to_string(),
            status: CheckStatus::Fail,
            severity: Severity::Error,
            summary: "one or more districts are disconnected".to_string(),
            witnesses,
        }
    }
}

fn district_components(
    members: &[usize],
    adjacency: &[Vec<rplan_core::UnitEdge>],
) -> Vec<Vec<usize>> {
    let member_set: BTreeSet<usize> = members.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut components = Vec::new();
    for &start in members {
        if seen.contains(&start) {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut component = Vec::new();
        seen.insert(start);
        while let Some(node) = queue.pop_front() {
            component.push(node);
            for edge in &adjacency[node] {
                let next = edge.to as usize;
                if member_set.contains(&next) && seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        components.push(component);
    }
    components
}

fn check_splits(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
    profile: &LegalProfile,
) -> AuditCheck {
    let mut checks = Vec::new();
    if !matches!(profile.county_split_rule, SplitRule::NotEvaluated) {
        checks.push(("county", &profile.county_split_rule));
    }
    if !matches!(profile.municipal_split_rule, SplitRule::NotEvaluated) {
        checks.push(("municipal", &profile.municipal_split_rule));
    }
    if checks.is_empty() {
        return not_evaluated_check(
            "splits",
            "subdivision split reporting is not requested by the supplied profile",
        );
    }

    let Some(subdivisions) = context.and_then(|ctx| ctx.subdivisions.as_ref()) else {
        return missing_input_check(
            "splits",
            "subdivisions",
            "split audit requires context subdivision memberships",
        );
    };

    let mut witnesses = Vec::new();
    let mut unsupported = Vec::new();
    for (kind, rule) in checks {
        if !matches!(rule, SplitRule::CountOnly) {
            unsupported.push(kind);
            continue;
        }
        let Some(ids) = subdivision_ids(subdivisions, kind) else {
            return missing_input_check(
                "splits",
                &format!("subdivisions.{kind}_ids"),
                &format!("split audit requires {kind} memberships"),
            );
        };
        witnesses.extend(split_witnesses(kind, ids, &plan.assignment));
    }

    if !unsupported.is_empty() {
        return AuditCheck {
            name: "splits".to_string(),
            status: CheckStatus::NotEvaluated,
            severity: Severity::Warning,
            summary: format!(
                "split rules are not implemented for subdivision kinds {:?}",
                unsupported
            ),
            witnesses,
        };
    }

    let split_count = witnesses.len();
    AuditCheck {
        name: "splits".to_string(),
        status: CheckStatus::Pass,
        severity: Severity::Info,
        summary: format!("{split_count} split subdivisions reported"),
        witnesses,
    }
}

fn subdivision_ids<'a>(
    subdivisions: &'a SubdivisionContext,
    kind: &str,
) -> Option<&'a [Option<String>]> {
    match kind {
        "county" => subdivisions.county_ids.as_deref(),
        "municipal" => subdivisions.municipal_ids.as_deref(),
        _ => None,
    }
}

fn split_witnesses(
    kind: &str,
    subdivision_ids: &[Option<String>],
    assignment: &[u32],
) -> Vec<Witness> {
    let mut memberships: BTreeMap<String, (BTreeSet<u32>, usize)> = BTreeMap::new();
    for (idx, subdivision_id) in subdivision_ids.iter().enumerate() {
        let Some(subdivision_id) = subdivision_id else {
            continue;
        };
        let entry = memberships
            .entry(subdivision_id.clone())
            .or_insert_with(|| (BTreeSet::new(), 0));
        entry.0.insert(assignment[idx]);
        entry.1 += 1;
    }

    memberships
        .into_iter()
        .filter_map(|(subdivision_id, (district_ids, unit_count))| {
            (district_ids.len() > 1).then(|| {
                Witness::Split(SplitWitness {
                    subdivision_kind: kind.to_string(),
                    subdivision_id,
                    district_ids: district_ids.into_iter().collect(),
                    unit_count,
                })
            })
        })
        .collect()
}

fn check_vra(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
    profile: &LegalProfile,
) -> AuditCheck {
    let VraPolicy::ReportOpportunityDistricts {
        minority_group,
        vap_threshold,
    } = &profile.vra_policy
    else {
        return not_evaluated_check(
            "vra",
            "VRA opportunity reporting is not requested by the supplied profile",
        );
    };
    if !vap_threshold.is_finite() || *vap_threshold < 0.0 || *vap_threshold > 1.0 {
        return missing_input_check(
            "vra",
            "legal_profile.vra_policy.vap_threshold",
            "VRA opportunity threshold must be between 0 and 1",
        );
    }

    let Some(demographics) = context.and_then(|ctx| ctx.demographics.as_ref()) else {
        return missing_input_check(
            "vra",
            "demographics",
            "VRA opportunity reporting requires context demographics",
        );
    };
    let Some(total_vap) = demographics.total_vap.as_ref() else {
        return missing_input_check(
            "vra",
            "demographics.total_vap",
            "VRA opportunity reporting requires total VAP",
        );
    };
    let Some(minority_vap) = demographics.minority_vap.as_ref() else {
        return missing_input_check(
            "vra",
            "demographics.minority_vap",
            "VRA opportunity reporting requires minority VAP",
        );
    };

    let witnesses = vra_witnesses(
        plan,
        minority_group,
        *vap_threshold,
        total_vap,
        minority_vap,
    );
    let opportunity_count = witnesses
        .iter()
        .filter(|witness| {
            matches!(
                witness,
                Witness::Vra(VraWitness {
                    is_opportunity_district: true,
                    ..
                })
            )
        })
        .count();

    AuditCheck {
        name: "vra".to_string(),
        status: CheckStatus::Pass,
        severity: Severity::Info,
        summary: format!("{opportunity_count} VRA opportunity districts reported"),
        witnesses,
    }
}

fn check_geometry(plan: &DistrictPlan, context: Option<&RplanContext>) -> AuditCheck {
    let Some(geometry) = context.and_then(|ctx| ctx.geometry.as_ref()) else {
        return missing_input_check(
            "geometry",
            "geometry",
            "geometry audit requires context geometry metadata",
        );
    };
    let Some(unit_geometry_hashes) = geometry.unit_geometry_hashes.as_ref() else {
        return missing_input_check(
            "geometry",
            "geometry.unit_geometry_hashes",
            "geometry audit requires per-unit geometry hashes",
        );
    };

    let unit_count = plan.units.unit_ids.len();
    let mut failures = Vec::new();
    if unit_geometry_hashes.len() != unit_count {
        failures.push(format!(
            "unit geometry hash length {} does not match unit count {unit_count}",
            unit_geometry_hashes.len()
        ));
    }
    if unit_geometry_hashes
        .iter()
        .any(|hash| !is_sha256_hash(hash))
    {
        failures.push("one or more unit geometry hashes are not sha256 hashes".to_string());
    }

    let witness = Witness::Geometry(GeometryWitness {
        source_id: geometry.source_id.clone(),
        crs: geometry.crs.clone(),
        unit_geometry_hash_count: unit_geometry_hashes.len(),
    });
    if failures.is_empty() {
        AuditCheck {
            name: "geometry".to_string(),
            status: CheckStatus::Pass,
            severity: Severity::Info,
            summary: format!(
                "{} unit geometry hashes are aligned to the plan unit universe",
                unit_geometry_hashes.len()
            ),
            witnesses: vec![witness],
        }
    } else {
        AuditCheck {
            name: "geometry".to_string(),
            status: CheckStatus::Fail,
            severity: Severity::Error,
            summary: failures.join("; "),
            witnesses: vec![witness],
        }
    }
}

fn vra_witnesses(
    plan: &DistrictPlan,
    minority_group: &str,
    vap_threshold: f64,
    total_vap_by_unit: &[f64],
    minority_vap_by_unit: &[f64],
) -> Vec<Witness> {
    let mut district_total_vap = vec![0.0; plan.k];
    let mut district_minority_vap = vec![0.0; plan.k];
    for (idx, &district_id) in plan.assignment.iter().enumerate() {
        let district_idx = district_id as usize;
        if district_idx >= plan.k {
            continue;
        }
        district_total_vap[district_idx] += total_vap_by_unit[idx];
        district_minority_vap[district_idx] += minority_vap_by_unit[idx];
    }

    (0..plan.k)
        .map(|district_id| {
            let total_vap = district_total_vap[district_id];
            let minority_vap = district_minority_vap[district_id];
            let minority_vap_percent = if total_vap > 0.0 {
                minority_vap / total_vap * 100.0
            } else {
                0.0
            };
            Witness::Vra(VraWitness {
                district_id: district_id as u32,
                minority_group: minority_group.to_string(),
                total_vap,
                minority_vap,
                minority_vap_percent,
                threshold_percent: vap_threshold * 100.0,
                is_opportunity_district: total_vap > 0.0
                    && minority_vap / total_vap > vap_threshold,
            })
        })
        .collect()
}

fn is_sha256_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn missing_input_check(name: &str, input: &str, reason: &str) -> AuditCheck {
    AuditCheck {
        name: name.to_string(),
        status: CheckStatus::MissingInput,
        severity: Severity::Error,
        summary: reason.to_string(),
        witnesses: vec![Witness::MissingInput(MissingInputWitness {
            input: input.to_string(),
            reason: reason.to_string(),
        })],
    }
}

fn not_evaluated_check(name: &str, reason: &str) -> AuditCheck {
    AuditCheck {
        name: name.to_string(),
        status: CheckStatus::NotEvaluated,
        severity: Severity::Info,
        summary: reason.to_string(),
        witnesses: Vec::new(),
    }
}

fn provenance_warnings(context: Option<&RplanContext>) -> Vec<AuditWarning> {
    if context
        .map(|ctx| ctx.source_hashes.entries.is_empty())
        .unwrap_or(true)
    {
        vec![AuditWarning {
            code: "PROVENANCE_INCOMPLETE".to_string(),
            severity: Severity::Warning,
            message: "source hashes are incomplete".to_string(),
            affected_check: None,
        }]
    } else {
        Vec::new()
    }
}

fn result_for(checks: &[AuditCheck], warnings: &[AuditWarning]) -> AuditResult {
    if checks
        .iter()
        .any(|check| matches!(check.status, CheckStatus::Fail | CheckStatus::MissingInput))
    {
        AuditResult::Fail
    } else if warnings
        .iter()
        .any(|warning| warning.severity == Severity::Warning)
    {
        AuditResult::PassWithWarnings
    } else {
        AuditResult::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rplan_core::{
        CanonicalOrder, DemographicContext, EdgeKind, EdgeSemantics, GeometryContext,
        PlanUnitIndex, UnitEdge, UnitGraph, UnitKind, DISTRICT_PLAN_SCHEMA_VERSION, RCTX_VERSION,
    };
    use std::collections::BTreeMap;

    fn units() -> PlanUnitIndex {
        PlanUnitIndex {
            unit_kind: UnitKind::Tract,
            state: Some("WA".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: vec![
                "53001000100".to_string(),
                "53001000200".to_string(),
                "53001000300".to_string(),
                "53001000400".to_string(),
                "53001000500".to_string(),
            ],
            unit_universe_hash: "sha256:path5".to_string(),
            source_id: None,
        }
    }

    fn plan(assignment: Vec<u32>) -> DistrictPlan {
        DistrictPlan {
            schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: units(),
            assignment,
            k: 2,
            display_labels: vec!["1".to_string(), "2".to_string()],
            allow_empty_districts: false,
        }
    }

    fn context(populations: Option<Vec<i64>>, graph: Option<UnitGraph>) -> RplanContext {
        RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: "sha256:ctx".to_string(),
            units: units(),
            graph,
            populations,
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes {
                entries: BTreeMap::from([("adjacency".to_string(), "sha256:abc".to_string())]),
            },
        }
    }

    fn context_with_subdivisions(
        populations: Option<Vec<i64>>,
        graph: Option<UnitGraph>,
        subdivisions: SubdivisionContext,
    ) -> RplanContext {
        RplanContext {
            subdivisions: Some(subdivisions),
            ..context(populations, graph)
        }
    }

    fn context_with_demographics(
        populations: Option<Vec<i64>>,
        graph: Option<UnitGraph>,
        demographics: DemographicContext,
    ) -> RplanContext {
        RplanContext {
            demographics: Some(demographics),
            ..context(populations, graph)
        }
    }

    fn context_with_geometry(
        populations: Option<Vec<i64>>,
        graph: Option<UnitGraph>,
        geometry: GeometryContext,
    ) -> RplanContext {
        RplanContext {
            geometry: Some(geometry),
            ..context(populations, graph)
        }
    }

    fn path_graph() -> UnitGraph {
        UnitGraph {
            edge_semantics: EdgeSemantics::Undirected,
            adjacency: vec![
                vec![UnitEdge {
                    to: 1,
                    kind: EdgeKind::Boundary,
                    weight: None,
                }],
                vec![
                    UnitEdge {
                        to: 0,
                        kind: EdgeKind::Boundary,
                        weight: None,
                    },
                    UnitEdge {
                        to: 2,
                        kind: EdgeKind::Boundary,
                        weight: None,
                    },
                ],
                vec![
                    UnitEdge {
                        to: 1,
                        kind: EdgeKind::Boundary,
                        weight: None,
                    },
                    UnitEdge {
                        to: 3,
                        kind: EdgeKind::Boundary,
                        weight: None,
                    },
                ],
                vec![
                    UnitEdge {
                        to: 2,
                        kind: EdgeKind::Boundary,
                        weight: None,
                    },
                    UnitEdge {
                        to: 4,
                        kind: EdgeKind::Boundary,
                        weight: None,
                    },
                ],
                vec![UnitEdge {
                    to: 3,
                    kind: EdgeKind::Boundary,
                    weight: None,
                }],
            ],
        }
    }

    fn profile() -> LegalProfile {
        LegalProfile {
            population_tolerance: PopulationToleranceRule::Percent {
                max_deviation_percent: 25.0,
            },
            ..LegalProfile::us_congressional_project_v1(2020)
        }
    }

    fn vra_profile() -> LegalProfile {
        LegalProfile {
            vra_policy: VraPolicy::ReportOpportunityDistricts {
                minority_group: "coalition".to_string(),
                vap_threshold: 0.50,
            },
            ..profile()
        }
    }

    fn runtime() -> RuntimeProvenance {
        RuntimeProvenance {
            binary_name: "rplan-test".to_string(),
            binary_version: "0.1.0".to_string(),
            ..RuntimeProvenance::default()
        }
    }

    #[test]
    fn shipped_us_congressional_project_profile_matches_default() {
        let shipped: LegalProfile =
            serde_json::from_str(include_str!("../profiles/us-congressional-project-v1.json"))
                .unwrap();
        assert_eq!(shipped, LegalProfile::us_congressional_project_v1(2020));
        assert!(shipped.hash().unwrap().starts_with("sha256:"));
    }

    #[test]
    fn incomplete_state_house_profile_is_rejected() {
        let incomplete: LegalProfile = serde_json::from_str(include_str!(
            "../profiles/incomplete-state-house-profile.json"
        ))
        .unwrap();
        let err = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &incomplete,
            runtime(),
            &[AuditConstraint::PlanShape],
            "2026-05-10T00:00:00Z",
        )
        .unwrap_err();
        assert!(matches!(err, AuditError::MissingStateLegislativeProfile));
    }

    #[test]
    fn grid3x3_golden_certificates_verify() {
        let valid_plan =
            rplan_io::read_rplan_str(include_str!("../fixtures/grid3x3-valid.rplan")).unwrap();
        let disconnected_plan =
            rplan_io::read_rplan_str(include_str!("../fixtures/grid3x3-disconnected.rplan"))
                .unwrap();
        let context = rplan_io::read_rctx_str(include_str!("../fixtures/grid3x3.rctx")).unwrap();

        let valid_cert: AuditCertificate =
            serde_json::from_str(include_str!("../fixtures/grid3x3-valid-certificate.json"))
                .unwrap();
        assert_eq!(valid_cert.result, AuditResult::Pass);
        assert_eq!(
            certificate_content_hash(&valid_cert).unwrap(),
            valid_cert.content_hash
        );
        verify_audit_certificate(&valid_cert, Some(&valid_plan.plan), Some(&context)).unwrap();

        let disconnected_cert: AuditCertificate = serde_json::from_str(include_str!(
            "../fixtures/grid3x3-disconnected-certificate.json"
        ))
        .unwrap();
        assert_eq!(disconnected_cert.result, AuditResult::Fail);
        assert_eq!(
            certificate_content_hash(&disconnected_cert).unwrap(),
            disconnected_cert.content_hash
        );
        verify_audit_certificate(
            &disconnected_cert,
            Some(&disconnected_plan.plan),
            Some(&context),
        )
        .unwrap();

        let missing_cert: AuditCertificate = serde_json::from_str(include_str!(
            "../fixtures/grid3x3-missing-contiguity-certificate.json"
        ))
        .unwrap();
        assert_eq!(missing_cert.result, AuditResult::Fail);
        assert_eq!(
            certificate_content_hash(&missing_cert).unwrap(),
            missing_cert.content_hash
        );
        verify_audit_certificate(&missing_cert, Some(&valid_plan.plan), None).unwrap();
    }

    #[test]
    fn path_fixture_context_can_audit_contiguity() {
        let context =
            rplan_io::read_rctx_str(include_str!("../../rplan-io/src/fixtures/path5.rctx"))
                .unwrap();
        let mut fixture_plan = plan(vec![0, 0, 0, 1, 1]);
        fixture_plan.units = context.units.clone();
        let cert = audit_plan(
            &fixture_plan,
            Some(&context),
            &profile(),
            runtime(),
            &[AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        assert_eq!(cert.result, AuditResult::PassWithWarnings);
    }

    #[test]
    fn valid_path_plan_passes_with_context() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Population, AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        assert_eq!(cert.result, AuditResult::Pass);
        assert_eq!(cert.schema_version, AUDIT_CERTIFICATE_SCHEMA_VERSION);
        assert!(cert.content_hash.starts_with("sha256:"));
    }

    #[test]
    fn disconnected_path_plan_fails_contiguity() {
        let cert = audit_plan(
            &plan(vec![0, 1, 0, 1, 1]),
            Some(&context(Some(vec![100; 5]), Some(path_graph()))),
            &profile(),
            runtime(),
            &[AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        let contiguity = cert
            .checks
            .iter()
            .find(|check| check.name == "contiguity")
            .unwrap();
        assert_eq!(cert.result, AuditResult::Fail);
        assert_eq!(contiguity.status, CheckStatus::Fail);
        assert!(matches!(
            contiguity.witnesses[0],
            Witness::Contiguity(ContiguityWitness { district_id: 0, .. })
        ));
    }

    #[test]
    fn missing_context_graph_is_missing_input() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            None,
            &profile(),
            runtime(),
            &[AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        let contiguity = cert
            .checks
            .iter()
            .find(|check| check.name == "contiguity")
            .unwrap();
        assert_eq!(contiguity.status, CheckStatus::MissingInput);
        assert_eq!(cert.result, AuditResult::Fail);
    }

    #[test]
    fn population_failure_reports_witness() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(Some(vec![90, 1, 1, 1, 1]), Some(path_graph()))),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        let population = cert
            .checks
            .iter()
            .find(|check| check.name == "population")
            .unwrap();
        assert_eq!(population.status, CheckStatus::Fail);
        assert!(!population.witnesses.is_empty());
    }

    #[test]
    fn requested_splits_without_subdivision_context_is_missing_input() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Splits, AuditConstraint::Vra],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            cert.checks
                .iter()
                .find(|check| check.name == "splits")
                .unwrap()
                .status,
            CheckStatus::MissingInput
        );
        assert_eq!(cert.result, AuditResult::Fail);
        assert_eq!(
            cert.checks
                .iter()
                .find(|check| check.name == "vra")
                .unwrap()
                .status,
            CheckStatus::NotEvaluated
        );
    }

    #[test]
    fn county_split_count_reports_split_witnesses() {
        let cert = audit_plan(
            &plan(vec![0, 0, 1, 1, 1]),
            Some(&context_with_subdivisions(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
                SubdivisionContext {
                    county_ids: Some(vec![
                        Some("county-a".to_string()),
                        Some("county-a".to_string()),
                        Some("county-a".to_string()),
                        Some("county-b".to_string()),
                        Some("county-b".to_string()),
                    ]),
                    municipal_ids: None,
                },
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Splits],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let splits = cert
            .checks
            .iter()
            .find(|check| check.name == "splits")
            .unwrap();
        assert_eq!(splits.status, CheckStatus::Pass);
        assert_eq!(splits.summary, "1 split subdivisions reported");
        assert!(matches!(
            &splits.witnesses[0],
            Witness::Split(SplitWitness {
                subdivision_kind,
                subdivision_id,
                district_ids,
                unit_count: 3,
            }) if subdivision_kind == "county"
                && subdivision_id == "county-a"
                && district_ids == &vec![0, 1]
        ));
    }

    #[test]
    fn vra_reporting_without_demographics_is_missing_input() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &vra_profile(),
            runtime(),
            &[AuditConstraint::Vra],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let vra = cert
            .checks
            .iter()
            .find(|check| check.name == "vra")
            .unwrap();
        assert_eq!(vra.status, CheckStatus::MissingInput);
        assert_eq!(cert.result, AuditResult::Fail);
    }

    #[test]
    fn vra_reporting_emits_district_vap_witnesses() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context_with_demographics(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
                DemographicContext {
                    total_vap: Some(vec![100.0, 100.0, 100.0, 100.0, 100.0]),
                    minority_vap: Some(vec![80.0, 80.0, 20.0, 40.0, 40.0]),
                },
            )),
            &vra_profile(),
            runtime(),
            &[AuditConstraint::Vra],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let vra = cert
            .checks
            .iter()
            .find(|check| check.name == "vra")
            .unwrap();
        assert_eq!(vra.status, CheckStatus::Pass);
        assert_eq!(vra.summary, "1 VRA opportunity districts reported");
        assert_eq!(vra.witnesses.len(), 2);
        assert!(matches!(
            &vra.witnesses[0],
            Witness::Vra(VraWitness {
                district_id: 0,
                minority_group,
                total_vap: 300.0,
                minority_vap: 180.0,
                is_opportunity_district: true,
                ..
            }) if minority_group == "coalition"
        ));
    }

    #[test]
    fn geometry_audit_without_geometry_context_is_missing_input() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(Some(vec![100; 5]), Some(path_graph()))),
            &profile(),
            runtime(),
            &[AuditConstraint::Geometry],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let geometry = cert
            .checks
            .iter()
            .find(|check| check.name == "geometry")
            .unwrap();
        assert_eq!(geometry.status, CheckStatus::MissingInput);
        assert_eq!(cert.result, AuditResult::Fail);
    }

    #[test]
    fn geometry_audit_reports_aligned_unit_hashes() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context_with_geometry(
                Some(vec![100; 5]),
                Some(path_graph()),
                GeometryContext {
                    source_id: Some("tiger-line-2020".to_string()),
                    crs: Some("EPSG:4326".to_string()),
                    unit_geometry_hashes: Some(vec![
                        format!("sha256:{}", "1".repeat(64)),
                        format!("sha256:{}", "2".repeat(64)),
                        format!("sha256:{}", "3".repeat(64)),
                        format!("sha256:{}", "4".repeat(64)),
                        format!("sha256:{}", "5".repeat(64)),
                    ]),
                },
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Geometry],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let geometry = cert
            .checks
            .iter()
            .find(|check| check.name == "geometry")
            .unwrap();
        assert_eq!(geometry.status, CheckStatus::Pass);
        assert!(geometry.summary.contains("5 unit geometry hashes"));
        assert!(matches!(
            &geometry.witnesses[0],
            Witness::Geometry(GeometryWitness {
                source_id: Some(source_id),
                crs: Some(crs),
                unit_geometry_hash_count: 5,
            }) if source_id == "tiger-line-2020" && crs == "EPSG:4326"
        ));
    }

    #[test]
    fn algorithm_lineage_round_trips_and_affects_content_hash() {
        let lineage = AlgorithmLineage::new(
            "bisect-ilp",
            "0.1.0",
            "branch-and-cut",
            vec![],
            serde_json::json!({
                "solve_report_dir": "intermediate/ilp_solve_reports",
                "solve_report_count": 1
            }),
        )
        .unwrap();
        assert!(lineage.parameters_hash.starts_with("sha256:"));
        let cert_with_lineage = audit_plan_with_lineage(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
            Some(lineage),
        )
        .unwrap();
        let cert_without_lineage = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            cert_with_lineage
                .algorithm_lineage
                .as_ref()
                .unwrap()
                .producer_crate,
            "bisect-ilp"
        );
        assert_ne!(
            cert_with_lineage.content_hash,
            cert_without_lineage.content_hash
        );
        let json = serde_json::to_string(&cert_with_lineage).unwrap();
        let decoded: AuditCertificate = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.algorithm_lineage.unwrap().method, "branch-and-cut");
    }

    #[test]
    fn algorithm_lineage_builder_rejects_reserved_certificate_fields() {
        let err = AlgorithmLineage::new(
            "mock-future-crate",
            "0.1.0",
            "mock-search",
            vec![],
            serde_json::json!({
                "plan_hash": "sha256:attempted-override"
            }),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AuditError::AlgorithmLineageExtraReservedField(field) if field == "plan_hash"
        ));
    }

    #[test]
    fn algorithm_lineage_extra_rejects_reserved_certificate_fields() {
        let lineage = AlgorithmLineage {
            producer_crate: "mock-future-crate".to_string(),
            producer_version: "0.1.0".to_string(),
            method: "mock-search".to_string(),
            parent_plan_hashes: vec![],
            parameters_hash: "sha256:params".to_string(),
            extra: serde_json::json!({
                "plan_hash": "sha256:attempted-override"
            }),
        };

        let err = audit_plan_with_lineage(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
            Some(lineage),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            AuditError::AlgorithmLineageExtraReservedField(field) if field == "plan_hash"
        ));
    }

    #[test]
    fn verify_rejects_certificate_with_reserved_lineage_extra() {
        let plan = plan(vec![0, 0, 0, 1, 1]);
        let mut context = context(Some(vec![100, 100, 100, 150, 150]), Some(path_graph()));
        context.context_hash = context.compute_context_hash().unwrap();
        let lineage = AlgorithmLineage {
            producer_crate: "mock-future-crate".to_string(),
            producer_version: "0.1.0".to_string(),
            method: "mock-search".to_string(),
            parent_plan_hashes: vec![],
            parameters_hash: "sha256:params".to_string(),
            extra: serde_json::json!({
                "branch_count": 2
            }),
        };
        let mut cert = audit_plan_with_lineage(
            &plan,
            Some(&context),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
            Some(lineage),
        )
        .unwrap();
        cert.algorithm_lineage.as_mut().unwrap().extra = serde_json::json!({
            "source_hashes": {
                "fixture": "sha256:attempted-override"
            }
        });
        cert.content_hash = certificate_content_hash(&cert).unwrap();
        cert.certificate_id = cert.content_hash.clone();

        let err = verify_audit_certificate(&cert, Some(&plan), Some(&context)).unwrap_err();
        assert!(matches!(
            err,
            AuditError::AlgorithmLineageExtraReservedField(field) if field == "source_hashes"
        ));
    }

    #[test]
    fn v1_reader_ignores_unknown_optional_certificate_fields() {
        let plan = plan(vec![0, 0, 0, 1, 1]);
        let mut context = context(Some(vec![100, 100, 100, 150, 150]), Some(path_graph()));
        context.context_hash = context.compute_context_hash().unwrap();
        let cert = audit_plan(
            &plan,
            Some(&context),
            &profile(),
            runtime(),
            &[AuditConstraint::Population, AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        let mut value = serde_json::to_value(&cert).unwrap();
        value.as_object_mut().unwrap().insert(
            "future_optional_field".to_string(),
            serde_json::json!({ "schema": "future-v2", "ignored_by_v1": true }),
        );

        let decoded: AuditCertificate = serde_json::from_value(value).unwrap();
        let verification = verify_audit_certificate(&decoded, Some(&plan), Some(&context)).unwrap();
        assert_eq!(verification.content_hash, cert.content_hash);
    }

    #[test]
    fn v1_reader_rejects_unknown_check_status() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        let mut value = serde_json::to_value(&cert).unwrap();
        value["checks"][0]["status"] = serde_json::Value::String("future-status".to_string());

        let err = serde_json::from_value::<AuditCertificate>(value).unwrap_err();
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn v1_reader_rejects_unknown_severity() {
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        let mut value = serde_json::to_value(&cert).unwrap();
        value["checks"][0]["severity"] = serde_json::Value::String("future-severity".to_string());

        let err = serde_json::from_value::<AuditCertificate>(value).unwrap_err();
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn exact_absolute_uses_total_deviation() {
        let mut profile = profile();
        profile.population_tolerance = PopulationToleranceRule::ExactAbsolute {
            max_total_deviation: 75,
        };
        let cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 50, 175, 175]),
                Some(path_graph()),
            )),
            &profile,
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            cert.checks
                .iter()
                .find(|check| check.name == "population")
                .unwrap()
                .status,
            CheckStatus::Fail
        );
    }

    #[test]
    fn content_hash_ignores_id_and_time() {
        let mut cert = audit_plan(
            &plan(vec![0, 0, 0, 1, 1]),
            Some(&context(
                Some(vec![100, 100, 100, 150, 150]),
                Some(path_graph()),
            )),
            &profile(),
            runtime(),
            &[AuditConstraint::Population, AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        let first = cert.content_hash.clone();
        cert.certificate_id = "different".to_string();
        cert.generated_at_utc = "2026-05-11T00:00:00Z".to_string();
        cert.content_hash = "different".to_string();
        assert_eq!(first, certificate_content_hash(&cert).unwrap());
    }

    #[test]
    fn verify_audit_certificate_accepts_matching_inputs() {
        let plan = plan(vec![0, 0, 0, 1, 1]);
        let mut context = context(Some(vec![100, 100, 100, 150, 150]), Some(path_graph()));
        context.context_hash = context.compute_context_hash().unwrap();
        let cert = audit_plan(
            &plan,
            Some(&context),
            &profile(),
            runtime(),
            &[AuditConstraint::Population, AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let verification = verify_audit_certificate(&cert, Some(&plan), Some(&context)).unwrap();
        assert_eq!(verification.content_hash, cert.content_hash);
        assert_eq!(verification.plan_hash, cert.plan_hash);
        assert_eq!(verification.context_hash, cert.context_hash);
    }

    #[test]
    fn verify_audit_certificate_rejects_tampered_content_hash() {
        let plan = plan(vec![0, 0, 0, 1, 1]);
        let mut context = context(Some(vec![100, 100, 100, 150, 150]), Some(path_graph()));
        context.context_hash = context.compute_context_hash().unwrap();
        let mut cert = audit_plan(
            &plan,
            Some(&context),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();
        cert.content_hash = "sha256:bad".to_string();

        let err = verify_audit_certificate(&cert, Some(&plan), Some(&context)).unwrap_err();
        assert!(matches!(
            err,
            AuditError::CertificateContentHashMismatch { .. }
        ));
    }

    #[test]
    fn verify_audit_certificate_rejects_wrong_plan() {
        let certified_plan = plan(vec![0, 0, 0, 1, 1]);
        let other_plan = plan(vec![0, 1, 1, 1, 0]);
        let mut context = context(Some(vec![100, 100, 100, 150, 150]), Some(path_graph()));
        context.context_hash = context.compute_context_hash().unwrap();
        let cert = audit_plan(
            &certified_plan,
            Some(&context),
            &profile(),
            runtime(),
            &[AuditConstraint::Population],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let err = verify_audit_certificate(&cert, Some(&other_plan), Some(&context)).unwrap_err();
        assert!(matches!(
            err,
            AuditError::CertificatePlanHashMismatch { .. }
        ));
    }

    #[test]
    fn verify_audit_certificate_rejects_missing_context_for_contextual_certificate() {
        let plan = plan(vec![0, 0, 0, 1, 1]);
        let mut context = context(Some(vec![100, 100, 100, 150, 150]), Some(path_graph()));
        context.context_hash = context.compute_context_hash().unwrap();
        let cert = audit_plan(
            &plan,
            Some(&context),
            &profile(),
            runtime(),
            &[AuditConstraint::Population, AuditConstraint::Contiguity],
            "2026-05-10T00:00:00Z",
        )
        .unwrap();

        let err = verify_audit_certificate(&cert, Some(&plan), None).unwrap_err();
        assert!(matches!(err, AuditError::CertificateContextMissing(_)));
    }
}
