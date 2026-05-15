use rplan_core::{
    CanonicalOrder, DistrictPlan, PlanUnitIndex, RplanContext, UnitKind,
    DISTRICT_PLAN_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RPLAN_V02: &str = "0.2";

#[derive(Debug, Error)]
pub enum RplanIoError {
    #[error("unsupported RPLAN version: {0}")]
    UnsupportedVersion(String),
    #[error("invalid v0.1 metadata year: {0}")]
    InvalidYear(String),
    #[error("v0.1 district id {district_id} is outside 1..={k}")]
    InvalidV01DistrictId { district_id: usize, k: usize },
    #[error("context_hash mismatch: declared {declared}, computed {computed}")]
    ContextHashMismatch { declared: String, computed: String },
    #[error("core error: {0}")]
    Core(#[from] rplan_core::RplanCoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RplanDocument {
    pub rplan_version: String,
    pub plan: DistrictPlan,
    pub metadata: RplanMetadataV02,
    pub provenance: RplanProvenance,
    pub geometry: Option<Value>,
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RplanMetadataV02 {
    pub label: String,
    pub jurisdiction: String,
    pub chamber: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RplanProvenance {
    #[serde(default)]
    pub producer: BTreeMap<String, Value>,
    #[serde(default)]
    pub source_hashes: BTreeMap<String, String>,
    #[serde(default)]
    pub conversion_lineage: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct RplanV01 {
    rplan_version: String,
    metadata: RplanMetadataV01,
    assignments: BTreeMap<String, usize>,
    geometry: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct RplanMetadataV01 {
    label: String,
    state_code: String,
    year: String,
    chamber: String,
    num_districts: usize,
    created_at: String,
    source_manifest: Option<Value>,
}

pub fn read_rplan_str(input: &str) -> Result<RplanDocument, RplanIoError> {
    let raw: Value = serde_json::from_str(input)?;
    let version = raw
        .get("rplan_version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match version {
        "0.2" => {
            let document: RplanDocument = serde_json::from_value(raw)?;
            document.plan.validate()?;
            Ok(document)
        }
        "0.1" => {
            let document: RplanV01 = serde_json::from_value(raw)?;
            convert_v01(document)
        }
        other => Err(RplanIoError::UnsupportedVersion(other.to_string())),
    }
}

pub fn write_rplan_string(document: &RplanDocument) -> Result<String, RplanIoError> {
    document.plan.validate()?;
    Ok(serde_json::to_string_pretty(document)?)
}

pub fn read_rctx_str(input: &str) -> Result<RplanContext, RplanIoError> {
    let context: RplanContext = serde_json::from_str(input)?;
    context.validate()?;
    let computed = context.compute_context_hash()?;
    if context.context_hash != computed {
        return Err(RplanIoError::ContextHashMismatch {
            declared: context.context_hash,
            computed,
        });
    }
    Ok(context)
}

pub fn write_rctx_string(context: &RplanContext) -> Result<String, RplanIoError> {
    let mut context = context.clone();
    context.validate()?;
    context.context_hash = context.compute_context_hash()?;
    Ok(serde_json::to_string_pretty(&context)?)
}

fn convert_v01(input: RplanV01) -> Result<RplanDocument, RplanIoError> {
    if input.rplan_version != "0.1" {
        return Err(RplanIoError::UnsupportedVersion(input.rplan_version));
    }
    let year: u16 = input
        .metadata
        .year
        .parse()
        .map_err(|_| RplanIoError::InvalidYear(input.metadata.year.clone()))?;
    let unit_ids: Vec<String> = input.assignments.keys().cloned().collect();
    let mut units = PlanUnitIndex {
        unit_kind: UnitKind::Tract,
        state: Some(input.metadata.state_code.clone()),
        year: Some(year),
        canonical_order: CanonicalOrder::SortedGeoid,
        unit_ids,
        unit_universe_hash: String::new(),
        source_id: None,
    };
    units.unit_universe_hash = units.compute_unit_universe_hash()?;

    let mut assignment = Vec::with_capacity(units.unit_ids.len());
    let mut labels = BTreeSet::new();
    for unit_id in &units.unit_ids {
        let district_id = input.assignments[unit_id];
        if district_id == 0 || district_id > input.metadata.num_districts {
            return Err(RplanIoError::InvalidV01DistrictId {
                district_id,
                k: input.metadata.num_districts,
            });
        }
        labels.insert(district_id);
        assignment.push((district_id - 1) as u32);
    }

    let display_labels = if labels.is_empty() {
        (1..=input.metadata.num_districts)
            .map(|district_id| district_id.to_string())
            .collect()
    } else {
        labels
            .into_iter()
            .map(|district_id| district_id.to_string())
            .collect()
    };

    let plan = DistrictPlan {
        schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units,
        assignment,
        k: input.metadata.num_districts,
        display_labels,
        allow_empty_districts: false,
    };
    plan.validate()?;

    let mut extensions = BTreeMap::new();
    if let Some(source_manifest) = input.metadata.source_manifest {
        extensions.insert(
            "org.bisect.rplan-v0_1.source_manifest".to_string(),
            source_manifest,
        );
    }

    Ok(RplanDocument {
        rplan_version: RPLAN_V02.to_string(),
        plan,
        metadata: RplanMetadataV02 {
            label: input.metadata.label,
            jurisdiction: input.metadata.state_code,
            chamber: input.metadata.chamber,
            created_at: input.metadata.created_at,
            description: None,
        },
        provenance: RplanProvenance::default(),
        geometry: input.geometry,
        extensions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rplan_core::{EdgeKind, EdgeSemantics, SourceHashes, UnitEdge, UnitGraph, RCTX_VERSION};

    #[test]
    fn reads_v02_plan() {
        let json = r#"{
          "rplan_version": "0.2",
          "plan": {
            "schema_version": "district-plan-v1",
            "units": {
              "unit_kind": "tract",
              "state": "WA",
              "year": 2020,
              "canonical_order": "explicit-unit-ids",
              "unit_ids": ["53001000100", "53001000200"],
              "unit_universe_hash": "sha256:test"
            },
            "assignment": [0, 1],
            "k": 2,
            "display_labels": ["1", "2"],
            "allow_empty_districts": false
          },
          "metadata": {
            "label": "wa_test",
            "jurisdiction": "WA",
            "chamber": "congressional",
            "created_at": "2026-05-10T00:00:00Z"
          },
          "provenance": {},
          "geometry": null,
          "extensions": {}
        }"#;
        let document = read_rplan_str(json).unwrap();
        assert_eq!(document.rplan_version, "0.2");
        assert_eq!(document.plan.assignment, vec![0, 1]);
    }

    #[test]
    fn converts_v01_to_v02_internal_ids() {
        let json = r#"{
          "rplan_version": "0.1",
          "metadata": {
            "label": "wa_test",
            "state_fips": "53",
            "state_code": "WA",
            "year": "2020",
            "chamber": "congressional",
            "num_districts": 2,
            "population_source": "total",
            "balance_tolerance_pct": 0.5,
            "created_at": "2026-04-26T00:00:00Z",
            "created_by": "test"
          },
          "assignments": {
            "53001000200": 2,
            "53001000100": 1
          },
          "geometry": null
        }"#;
        let document = read_rplan_str(json).unwrap();
        assert_eq!(document.rplan_version, "0.2");
        assert_eq!(
            document.plan.units.unit_ids,
            vec!["53001000100", "53001000200"]
        );
        assert_eq!(document.plan.assignment, vec![0, 1]);
        assert_eq!(document.plan.display_labels, vec!["1", "2"]);
    }

    #[test]
    fn preserves_v01_source_manifest_as_extension() {
        let json = r#"{
          "rplan_version": "0.1",
          "metadata": {
            "label": "wa_test",
            "state_fips": "53",
            "state_code": "WA",
            "year": "2020",
            "chamber": "congressional",
            "num_districts": 1,
            "population_source": "total",
            "balance_tolerance_pct": 0.5,
            "created_at": "2026-04-26T00:00:00Z",
            "created_by": "test",
            "source_manifest": {"manifest_version": "test-v1"}
          },
          "assignments": {
            "53001000100": 1
          },
          "geometry": null
        }"#;
        let document = read_rplan_str(json).unwrap();
        assert_eq!(
            document.extensions["org.bisect.rplan-v0_1.source_manifest"]["manifest_version"],
            "test-v1"
        );
    }

    #[test]
    fn round_trips_path_context() {
        let mut units = PlanUnitIndex {
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
            unit_universe_hash: String::new(),
            source_id: None,
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        let context = RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units,
            graph: Some(UnitGraph {
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
            }),
            populations: Some(vec![100, 100, 100, 100, 100]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes::default(),
        };
        let json = write_rctx_string(&context).unwrap();
        let decoded = read_rctx_str(&json).unwrap();
        assert_eq!(decoded.populations.unwrap(), vec![100, 100, 100, 100, 100]);
    }

    #[test]
    fn round_trips_path_context_fixture() {
        let fixture = include_str!("fixtures/path5.rctx");
        let context = read_rctx_str(fixture).unwrap();
        let json = write_rctx_string(&context).unwrap();
        let decoded = read_rctx_str(&json).unwrap();
        assert_eq!(decoded.units.unit_ids.len(), 5);
        assert_eq!(decoded.graph.unwrap().adjacency.len(), 5);
    }

    #[test]
    fn rejects_stale_context_hash() {
        let fixture = include_str!("fixtures/path5.rctx").replace(
            "51020b03156366231a546028d34c4439ad9f14a716ccbf55fd62c2ea150d843f",
            "0000",
        );
        assert!(matches!(
            read_rctx_str(&fixture),
            Err(RplanIoError::ContextHashMismatch { .. })
        ));
    }
}
