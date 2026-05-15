use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const DISTRICT_PLAN_SCHEMA_VERSION: &str = "district-plan-v1";
pub const RCTX_VERSION: &str = "0.1";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RplanCoreError {
    #[error("assignment length {assignment_len} does not match unit count {unit_count}")]
    AssignmentLengthMismatch {
        assignment_len: usize,
        unit_count: usize,
    },
    #[error("district id {district_id} is outside canonical range 0..{k}")]
    InvalidDistrictId { district_id: u32, k: usize },
    #[error("district {district_id} is empty")]
    EmptyDistrict { district_id: u32 },
    #[error("district count k must be greater than zero")]
    InvalidDistrictCount,
    #[error("unit id at index {index} is invalid for {unit_kind:?}: {unit_id}")]
    InvalidUnitId {
        unit_kind: UnitKind,
        index: usize,
        unit_id: String,
    },
    #[error("duplicate unit id: {0}")]
    DuplicateUnitId(String),
    #[error("unit ids are not sorted for sorted-geoid order")]
    UnitIdsNotSorted,
    #[error("adjacency length {adjacency_len} does not match unit count {unit_count}")]
    AdjacencyLengthMismatch {
        adjacency_len: usize,
        unit_count: usize,
    },
    #[error("population length {population_len} does not match unit count {unit_count}")]
    PopulationLengthMismatch {
        population_len: usize,
        unit_count: usize,
    },
    #[error("{subdivision_kind} subdivision length {subdivision_len} does not match unit count {unit_count}")]
    SubdivisionLengthMismatch {
        subdivision_kind: String,
        subdivision_len: usize,
        unit_count: usize,
    },
    #[error("{demographic_kind} demographic length {demographic_len} does not match unit count {unit_count}")]
    DemographicLengthMismatch {
        demographic_kind: String,
        demographic_len: usize,
        unit_count: usize,
    },
    #[error("demographic value must be finite and non-negative")]
    InvalidDemographicValue,
    #[error(
        "unit geometry hash length {geometry_hash_len} does not match unit count {unit_count}"
    )]
    GeometryHashLengthMismatch {
        geometry_hash_len: usize,
        unit_count: usize,
    },
    #[error("unit geometry hash is invalid: {0}")]
    InvalidGeometryHash(String),
    #[error("edge target {to} at vertex {from} is outside unit range")]
    InvalidEdgeTarget { from: usize, to: u32 },
    #[error("duplicate edge from {from} to {to}")]
    DuplicateEdge { from: usize, to: u32 },
    #[error("missing symmetric edge from {from} to {to}")]
    MissingSymmetricEdge { from: usize, to: u32 },
    #[error("edge weight must be finite and non-negative")]
    InvalidEdgeWeight,
    #[error("canonical JSON error: {0}")]
    CanonicalJson(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitKind {
    Block,
    BlockGroup,
    Tract,
    County,
    Precinct,
    Imported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonicalOrder {
    ExplicitUnitIds,
    SortedGeoid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanUnitIndex {
    pub unit_kind: UnitKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    pub canonical_order: CanonicalOrder,
    pub unit_ids: Vec<String>,
    pub unit_universe_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictPlan {
    pub schema_version: String,
    pub units: PlanUnitIndex,
    pub assignment: Vec<u32>,
    pub k: usize,
    pub display_labels: Vec<String>,
    pub allow_empty_districts: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeKind {
    Boundary,
    PointTouch,
    Bridge,
    Ferry,
    Water,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitEdge {
    pub to: u32,
    pub kind: EdgeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitGraph {
    pub edge_semantics: EdgeSemantics,
    pub adjacency: Vec<Vec<UnitEdge>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeSemantics {
    Undirected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceHashes {
    #[serde(flatten)]
    pub entries: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SubdivisionContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub county_ids: Option<Vec<Option<String>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub municipal_ids: Option<Vec<Option<String>>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DemographicContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_vap: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minority_vap: Option<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GeometryContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crs: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_geometry_hashes: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RplanContext {
    pub rctx_version: String,
    pub context_hash: String,
    pub units: PlanUnitIndex,
    pub graph: Option<UnitGraph>,
    pub populations: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subdivisions: Option<SubdivisionContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub demographics: Option<DemographicContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryContext>,
    pub source_hashes: SourceHashes,
}

impl PlanUnitIndex {
    pub fn validate(&self) -> Result<(), RplanCoreError> {
        validate_unique_unit_ids(&self.unit_ids)?;
        for (index, unit_id) in self.unit_ids.iter().enumerate() {
            if !valid_unit_id(self.unit_kind, unit_id) {
                return Err(RplanCoreError::InvalidUnitId {
                    unit_kind: self.unit_kind,
                    index,
                    unit_id: unit_id.clone(),
                });
            }
        }
        if self.canonical_order == CanonicalOrder::SortedGeoid && !is_sorted(&self.unit_ids) {
            return Err(RplanCoreError::UnitIdsNotSorted);
        }
        Ok(())
    }

    pub fn compute_unit_universe_hash(&self) -> Result<String, RplanCoreError> {
        let mut value = Map::new();
        value.insert("unit_kind".to_string(), to_value(self.unit_kind)?);
        if let Some(state) = &self.state {
            value.insert("state".to_string(), Value::String(state.clone()));
        }
        if let Some(year) = self.year {
            value.insert("year".to_string(), serde_json::json!(year));
        }
        value.insert(
            "canonical_order".to_string(),
            to_value(self.canonical_order)?,
        );
        value.insert("unit_ids".to_string(), to_value(&self.unit_ids)?);
        if let Some(source_id) = &self.source_id {
            value.insert("source_id".to_string(), Value::String(source_id.clone()));
        }
        canonical_sha256(&Value::Object(value))
    }
}

impl DistrictPlan {
    pub fn validate(&self) -> Result<(), RplanCoreError> {
        self.units.validate()?;
        if self.k == 0 {
            return Err(RplanCoreError::InvalidDistrictCount);
        }
        if self.assignment.len() != self.units.unit_ids.len() {
            return Err(RplanCoreError::AssignmentLengthMismatch {
                assignment_len: self.assignment.len(),
                unit_count: self.units.unit_ids.len(),
            });
        }
        let mut seen = vec![false; self.k];
        for &district_id in &self.assignment {
            let idx = district_id as usize;
            if idx >= self.k {
                return Err(RplanCoreError::InvalidDistrictId {
                    district_id,
                    k: self.k,
                });
            }
            seen[idx] = true;
        }
        if !self.allow_empty_districts {
            for (idx, present) in seen.iter().enumerate() {
                if !present {
                    return Err(RplanCoreError::EmptyDistrict {
                        district_id: idx as u32,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn plan_hash(&self) -> Result<String, RplanCoreError> {
        let mut units = Map::new();
        units.insert("unit_kind".to_string(), to_value(self.units.unit_kind)?);
        if let Some(state) = &self.units.state {
            units.insert("state".to_string(), Value::String(state.clone()));
        }
        if let Some(year) = self.units.year {
            units.insert("year".to_string(), serde_json::json!(year));
        }
        units.insert(
            "canonical_order".to_string(),
            to_value(self.units.canonical_order)?,
        );
        units.insert("unit_ids".to_string(), to_value(&self.units.unit_ids)?);
        units.insert(
            "unit_universe_hash".to_string(),
            Value::String(self.units.unit_universe_hash.clone()),
        );

        let mut value = Map::new();
        value.insert(
            "schema_version".to_string(),
            Value::String(self.schema_version.clone()),
        );
        value.insert("units".to_string(), Value::Object(units));
        value.insert("assignment".to_string(), to_value(&self.assignment)?);
        value.insert("k".to_string(), serde_json::json!(self.k));
        value.insert(
            "allow_empty_districts".to_string(),
            Value::Bool(self.allow_empty_districts),
        );
        canonical_sha256(&Value::Object(value))
    }
}

impl UnitGraph {
    pub fn validate(&self, unit_count: usize) -> Result<(), RplanCoreError> {
        if self.adjacency.len() != unit_count {
            return Err(RplanCoreError::AdjacencyLengthMismatch {
                adjacency_len: self.adjacency.len(),
                unit_count,
            });
        }
        for (from, edges) in self.adjacency.iter().enumerate() {
            let mut targets = BTreeSet::new();
            for edge in edges {
                if edge.to as usize >= unit_count {
                    return Err(RplanCoreError::InvalidEdgeTarget { from, to: edge.to });
                }
                if !targets.insert(edge.to) {
                    return Err(RplanCoreError::DuplicateEdge { from, to: edge.to });
                }
                if let Some(weight) = edge.weight {
                    if !weight.is_finite() || weight < 0.0 {
                        return Err(RplanCoreError::InvalidEdgeWeight);
                    }
                }
            }
        }
        for (from, edges) in self.adjacency.iter().enumerate() {
            for edge in edges {
                let reverse_exists = self.adjacency[edge.to as usize]
                    .iter()
                    .any(|candidate| candidate.to as usize == from);
                if !reverse_exists {
                    return Err(RplanCoreError::MissingSymmetricEdge {
                        from: edge.to as usize,
                        to: from as u32,
                    });
                }
            }
        }
        Ok(())
    }
}

impl RplanContext {
    pub fn validate(&self) -> Result<(), RplanCoreError> {
        self.units.validate()?;
        let unit_count = self.units.unit_ids.len();
        if let Some(graph) = &self.graph {
            graph.validate(unit_count)?;
        }
        if let Some(populations) = &self.populations {
            if populations.len() != unit_count {
                return Err(RplanCoreError::PopulationLengthMismatch {
                    population_len: populations.len(),
                    unit_count,
                });
            }
        }
        if let Some(subdivisions) = &self.subdivisions {
            subdivisions.validate(unit_count)?;
        }
        if let Some(demographics) = &self.demographics {
            demographics.validate(unit_count)?;
        }
        if let Some(geometry) = &self.geometry {
            geometry.validate(unit_count)?;
        }
        Ok(())
    }

    pub fn compute_context_hash(&self) -> Result<String, RplanCoreError> {
        let mut value = serde_json::json!({
            "units": self.units,
            "graph": self.graph,
            "populations": self.populations,
            "source_hashes": self.source_hashes,
        });
        if let Some(subdivisions) = &self.subdivisions {
            value
                .as_object_mut()
                .expect("context hash projection is an object")
                .insert("subdivisions".to_string(), to_value(subdivisions)?);
        }
        if let Some(demographics) = &self.demographics {
            value
                .as_object_mut()
                .expect("context hash projection is an object")
                .insert("demographics".to_string(), to_value(demographics)?);
        }
        if let Some(geometry) = &self.geometry {
            value
                .as_object_mut()
                .expect("context hash projection is an object")
                .insert("geometry".to_string(), to_value(geometry)?);
        }
        canonical_sha256(&value)
    }
}

impl SubdivisionContext {
    pub fn validate(&self, unit_count: usize) -> Result<(), RplanCoreError> {
        if let Some(county_ids) = &self.county_ids {
            if county_ids.len() != unit_count {
                return Err(RplanCoreError::SubdivisionLengthMismatch {
                    subdivision_kind: "county".to_string(),
                    subdivision_len: county_ids.len(),
                    unit_count,
                });
            }
        }
        if let Some(municipal_ids) = &self.municipal_ids {
            if municipal_ids.len() != unit_count {
                return Err(RplanCoreError::SubdivisionLengthMismatch {
                    subdivision_kind: "municipal".to_string(),
                    subdivision_len: municipal_ids.len(),
                    unit_count,
                });
            }
        }
        Ok(())
    }
}

impl DemographicContext {
    pub fn validate(&self, unit_count: usize) -> Result<(), RplanCoreError> {
        validate_demographic_series("total_vap", &self.total_vap, unit_count)?;
        validate_demographic_series("minority_vap", &self.minority_vap, unit_count)?;
        if let (Some(total_vap), Some(minority_vap)) = (&self.total_vap, &self.minority_vap) {
            if total_vap
                .iter()
                .zip(minority_vap)
                .any(|(total, minority)| minority > total)
            {
                return Err(RplanCoreError::InvalidDemographicValue);
            }
        }
        Ok(())
    }
}

impl GeometryContext {
    pub fn validate(&self, unit_count: usize) -> Result<(), RplanCoreError> {
        let Some(unit_geometry_hashes) = &self.unit_geometry_hashes else {
            return Ok(());
        };
        if unit_geometry_hashes.len() != unit_count {
            return Err(RplanCoreError::GeometryHashLengthMismatch {
                geometry_hash_len: unit_geometry_hashes.len(),
                unit_count,
            });
        }
        for hash in unit_geometry_hashes {
            if !is_sha256_hash(hash) {
                return Err(RplanCoreError::InvalidGeometryHash(hash.clone()));
            }
        }
        Ok(())
    }
}

fn validate_demographic_series(
    kind: &str,
    values: &Option<Vec<f64>>,
    unit_count: usize,
) -> Result<(), RplanCoreError> {
    let Some(values) = values else {
        return Ok(());
    };
    if values.len() != unit_count {
        return Err(RplanCoreError::DemographicLengthMismatch {
            demographic_kind: kind.to_string(),
            demographic_len: values.len(),
            unit_count,
        });
    }
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(RplanCoreError::InvalidDemographicValue);
    }
    Ok(())
}

fn is_sha256_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn valid_unit_id(kind: UnitKind, unit_id: &str) -> bool {
    match kind {
        UnitKind::Tract => fixed_ascii_digits(unit_id, 11),
        UnitKind::BlockGroup => fixed_ascii_digits(unit_id, 12),
        UnitKind::Block => fixed_ascii_digits(unit_id, 15),
        UnitKind::County => fixed_ascii_digits(unit_id, 5),
        UnitKind::Precinct | UnitKind::Imported => !unit_id.is_empty(),
    }
}

pub fn canonical_sha256(value: &Value) -> Result<String, RplanCoreError> {
    let canonical = canonical_json(value)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(format!("sha256:{}", lowercase_hex(&hasher.finalize())))
}

pub fn canonical_json(value: &Value) -> Result<String, RplanCoreError> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(number) => Ok(number.to_string()),
        Value::String(value) => serde_json::to_string(value)
            .map_err(|err| RplanCoreError::CanonicalJson(err.to_string())),
        Value::Array(values) => {
            let mut out = String::from("[");
            for (idx, item) in values.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                out.push_str(&canonical_json(item)?);
            }
            out.push(']');
            Ok(out)
        }
        Value::Object(map) => canonical_json_object(map),
    }
}

fn canonical_json_object(map: &Map<String, Value>) -> Result<String, RplanCoreError> {
    let mut sorted: BTreeMap<&String, &Value> = BTreeMap::new();
    for (key, value) in map {
        sorted.insert(key, value);
    }
    let mut out = String::from("{");
    for (idx, (key, value)) in sorted.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(
            &serde_json::to_string(key)
                .map_err(|err| RplanCoreError::CanonicalJson(err.to_string()))?,
        );
        out.push(':');
        out.push_str(&canonical_json(value)?);
    }
    out.push('}');
    Ok(out)
}

fn fixed_ascii_digits(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn validate_unique_unit_ids(unit_ids: &[String]) -> Result<(), RplanCoreError> {
    let mut seen = BTreeSet::new();
    for unit_id in unit_ids {
        if !seen.insert(unit_id) {
            return Err(RplanCoreError::DuplicateUnitId(unit_id.clone()));
        }
    }
    Ok(())
}

fn is_sorted(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] <= pair[1])
}

fn to_value<T: Serialize>(value: T) -> Result<Value, RplanCoreError> {
    serde_json::to_value(value).map_err(|err| RplanCoreError::CanonicalJson(err.to_string()))
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_units() -> PlanUnitIndex {
        let mut units = PlanUnitIndex {
            unit_kind: UnitKind::Tract,
            state: Some("WA".to_string()),
            year: Some(2020),
            canonical_order: CanonicalOrder::ExplicitUnitIds,
            unit_ids: vec!["53001000100".to_string(), "53001000200".to_string()],
            unit_universe_hash: String::new(),
            source_id: Some("tiger-line-2020".to_string()),
        };
        units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
        units
    }

    #[test]
    fn canonical_json_sorts_object_keys() {
        let a = serde_json::json!({"b": 1, "a": [true, null]});
        let b = serde_json::json!({"a": [true, null], "b": 1});
        assert_eq!(canonical_json(&a).unwrap(), "{\"a\":[true,null],\"b\":1}");
        assert_eq!(canonical_sha256(&a).unwrap(), canonical_sha256(&b).unwrap());
    }

    #[test]
    fn validates_unit_ids_by_kind() {
        assert!(valid_unit_id(UnitKind::Tract, "53001000100"));
        assert!(!valid_unit_id(UnitKind::Tract, "5300100010"));
        assert!(valid_unit_id(UnitKind::BlockGroup, "530010001001"));
        assert!(valid_unit_id(UnitKind::Block, "530010001001234"));
        assert!(valid_unit_id(UnitKind::County, "53001"));
        assert!(valid_unit_id(UnitKind::Precinct, "wa:precinct-1"));
        assert!(!valid_unit_id(UnitKind::Imported, ""));
    }

    #[test]
    fn display_labels_do_not_change_plan_hash() {
        let units = test_units();
        let mut plan = DistrictPlan {
            schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units,
            assignment: vec![0, 1],
            k: 2,
            display_labels: vec!["1".to_string(), "2".to_string()],
            allow_empty_districts: false,
        };
        let first = plan.plan_hash().unwrap();
        plan.display_labels = vec!["A".to_string(), "B".to_string()];
        assert_eq!(first, plan.plan_hash().unwrap());
    }

    #[test]
    fn assignment_changes_plan_hash() {
        let units = test_units();
        let mut plan = DistrictPlan {
            schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units,
            assignment: vec![0, 1],
            k: 2,
            display_labels: vec!["1".to_string(), "2".to_string()],
            allow_empty_districts: false,
        };
        let first = plan.plan_hash().unwrap();
        plan.assignment = vec![1, 0];
        assert_ne!(first, plan.plan_hash().unwrap());
    }

    #[test]
    fn zero_district_count_is_invalid() {
        let plan = DistrictPlan {
            schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: PlanUnitIndex {
                unit_kind: UnitKind::Imported,
                state: None,
                year: None,
                canonical_order: CanonicalOrder::ExplicitUnitIds,
                unit_ids: Vec::new(),
                unit_universe_hash: "sha256:test".to_string(),
                source_id: None,
            },
            assignment: Vec::new(),
            k: 0,
            display_labels: Vec::new(),
            allow_empty_districts: false,
        };
        assert_eq!(plan.validate(), Err(RplanCoreError::InvalidDistrictCount));
    }

    #[test]
    fn plan_hash_matches_v02_fixture() {
        let plan = DistrictPlan {
            schema_version: DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
            units: PlanUnitIndex {
                unit_kind: UnitKind::Tract,
                state: Some("WA".to_string()),
                year: Some(2020),
                canonical_order: CanonicalOrder::ExplicitUnitIds,
                unit_ids: vec!["53001000100".to_string(), "53001000200".to_string()],
                unit_universe_hash: "sha256:test".to_string(),
                source_id: None,
            },
            assignment: vec![0, 1],
            k: 2,
            display_labels: vec!["1".to_string(), "2".to_string()],
            allow_empty_districts: false,
        };
        assert_eq!(
            plan.plan_hash().unwrap(),
            "sha256:b4789f07775494224e44cb8702242ecda5a99a22388aa603d791715f617cc078"
        );
    }

    #[test]
    fn validates_symmetric_path_graph_context() {
        let units = test_units();
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
                        weight: Some(1.0),
                    }],
                    vec![UnitEdge {
                        to: 0,
                        kind: EdgeKind::Boundary,
                        weight: Some(1.0),
                    }],
                ],
            }),
            populations: Some(vec![100, 100]),
            subdivisions: None,
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes::default(),
        };
        context.validate().unwrap();
        assert!(context
            .compute_context_hash()
            .unwrap()
            .starts_with("sha256:"));
    }

    #[test]
    fn validates_subdivision_lengths() {
        let context = RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units: test_units(),
            graph: None,
            populations: None,
            subdivisions: Some(SubdivisionContext {
                county_ids: Some(vec![Some("001".to_string())]),
                municipal_ids: None,
            }),
            demographics: None,
            geometry: None,
            source_hashes: SourceHashes::default(),
        };

        assert!(matches!(
            context.validate(),
            Err(RplanCoreError::SubdivisionLengthMismatch {
                subdivision_kind,
                ..
            }) if subdivision_kind == "county"
        ));
    }

    #[test]
    fn validates_demographic_lengths_and_values() {
        let context = RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units: test_units(),
            graph: None,
            populations: None,
            subdivisions: None,
            demographics: Some(DemographicContext {
                total_vap: Some(vec![100.0, f64::NAN]),
                minority_vap: None,
            }),
            geometry: None,
            source_hashes: SourceHashes::default(),
        };
        assert_eq!(
            context.validate(),
            Err(RplanCoreError::InvalidDemographicValue)
        );
    }

    #[test]
    fn validates_geometry_hash_lengths_and_values() {
        let context = RplanContext {
            rctx_version: RCTX_VERSION.to_string(),
            context_hash: String::new(),
            units: test_units(),
            graph: None,
            populations: None,
            subdivisions: None,
            demographics: None,
            geometry: Some(GeometryContext {
                source_id: Some("fixture".to_string()),
                crs: Some("EPSG:4326".to_string()),
                unit_geometry_hashes: Some(vec!["not-a-hash".to_string()]),
            }),
            source_hashes: SourceHashes::default(),
        };
        assert!(matches!(
            context.validate(),
            Err(RplanCoreError::GeometryHashLengthMismatch { .. })
        ));

        let context = RplanContext {
            geometry: Some(GeometryContext {
                source_id: Some("fixture".to_string()),
                crs: Some("EPSG:4326".to_string()),
                unit_geometry_hashes: Some(vec![
                    format!("sha256:{}", "a".repeat(64)),
                    "not-a-hash".to_string(),
                ]),
            }),
            ..context
        };
        assert!(matches!(
            context.validate(),
            Err(RplanCoreError::InvalidGeometryHash(hash)) if hash == "not-a-hash"
        ));
    }
}
