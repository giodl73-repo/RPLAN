//! Portable forms of the original CLI chamber/profile applicability checks.
//! Applicability is a package contract check, not a determination of law.
use crate::{Chamber, LegalProfile};
use rplan_io::RplanDocument;

pub fn parse_plan_chamber(value: &str) -> Result<Chamber, String> {
    match value {
        "congressional" => Ok(Chamber::Congressional),
        "state-house" => Ok(Chamber::StateHouse),
        "state-senate" => Ok(Chamber::StateSenate),
        "local" => Ok(Chamber::Local),
        "custom" => Ok(Chamber::Custom("custom".to_string())),
        other => Err(format!("unknown chamber '{other}'")),
    }
}

pub fn validate_profile_applicability(
    document: &RplanDocument,
    plan_chamber: &Chamber,
    profile: &LegalProfile,
) -> Result<(), String> {
    if &profile.chamber != plan_chamber {
        return Err(format!(
            "legal profile chamber {:?} does not match plan chamber {:?}",
            profile.chamber, plan_chamber
        ));
    }
    if profile.jurisdiction != "US" && profile.jurisdiction != document.metadata.jurisdiction {
        return Err(format!(
            "legal profile jurisdiction '{}' does not match plan jurisdiction '{}'",
            profile.jurisdiction, document.metadata.jurisdiction
        ));
    }
    if let Some(plan_year) = document.plan.units.year {
        if profile.year != plan_year {
            return Err(format!(
                "legal profile year {} does not match plan year {}",
                profile.year, plan_year
            ));
        }
    }
    Ok(())
}
