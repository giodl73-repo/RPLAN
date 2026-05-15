use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rplan_audit::{
    audit_plan_with_lineage, verify_audit_certificate, AlgorithmLineage, AuditCertificate,
    AuditConstraint, AuditResult, Chamber, LegalProfile, RuntimeProvenance,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rplan")]
#[command(about = "RPLAN interchange and audit tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Audit(AuditArgs),
    VerifyCertificate(VerifyCertificateArgs),
}

#[derive(Debug, Parser)]
struct AuditArgs {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    context: Option<PathBuf>,
    #[arg(long)]
    legal_profile: Option<PathBuf>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "plan-shape,population,contiguity"
    )]
    constraints: Vec<ConstraintArg>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
    #[arg(long)]
    allow_warnings: bool,
    #[arg(long)]
    fixed_generated_at: Option<String>,
    #[arg(long)]
    lineage_producer_crate: Option<String>,
    #[arg(long)]
    lineage_producer_version: Option<String>,
    #[arg(long)]
    lineage_method: Option<String>,
    #[arg(long, value_delimiter = ',')]
    lineage_parent_plan_hash: Vec<String>,
    #[arg(long)]
    lineage_extra_json: Option<String>,
}

#[derive(Debug, Parser)]
struct VerifyCertificateArgs {
    #[arg(long)]
    certificate: PathBuf,
    #[arg(long)]
    plan: Option<PathBuf>,
    #[arg(long)]
    context: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ConstraintArg {
    PlanShape,
    Population,
    Contiguity,
    Splits,
    Vra,
    Geometry,
}

impl From<ConstraintArg> for AuditConstraint {
    fn from(value: ConstraintArg) -> Self {
        match value {
            ConstraintArg::PlanShape => AuditConstraint::PlanShape,
            ConstraintArg::Population => AuditConstraint::Population,
            ConstraintArg::Contiguity => AuditConstraint::Contiguity,
            ConstraintArg::Splits => AuditConstraint::Splits,
            ConstraintArg::Vra => AuditConstraint::Vra,
            ConstraintArg::Geometry => AuditConstraint::Geometry,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    PrettyJson,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    match Cli::parse().command {
        Commands::Audit(args) => run_audit(args),
        Commands::VerifyCertificate(args) => run_verify_certificate(args),
    }
}

fn run_audit(args: AuditArgs) -> Result<i32> {
    let plan_text = std::fs::read_to_string(&args.plan)
        .with_context(|| format!("reading plan {}", args.plan.display()))?;
    let document = rplan_io::read_rplan_str(&plan_text)
        .with_context(|| format!("parsing plan {}", args.plan.display()))?;

    let context = if let Some(path) = &args.context {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading context {}", path.display()))?;
        Some(
            rplan_io::read_rctx_str(&text)
                .with_context(|| format!("parsing context {}", path.display()))?,
        )
    } else {
        None
    };

    let plan_chamber = parse_chamber(&document.metadata.chamber)?;
    let profile = if let Some(path) = &args.legal_profile {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading legal profile {}", path.display()))?;
        let profile = serde_json::from_str::<LegalProfile>(&text)
            .with_context(|| format!("parsing legal profile {}", path.display()))?;
        validate_profile_applicability(&document, &plan_chamber, &profile)?;
        profile
    } else {
        if !matches!(plan_chamber, Chamber::Congressional) {
            anyhow::bail!(
                "--legal-profile is required for non-congressional chamber '{}'",
                document.metadata.chamber
            );
        }
        let year = document
            .plan
            .units
            .year
            .or_else(|| {
                document
                    .metadata
                    .created_at
                    .get(0..4)
                    .and_then(|year| year.parse().ok())
            })
            .unwrap_or(2020);
        LegalProfile::us_congressional_project_v1(year)
    };

    let constraints: Vec<AuditConstraint> = args
        .constraints
        .iter()
        .copied()
        .map(AuditConstraint::from)
        .collect();
    let runtime = RuntimeProvenance {
        binary_name: "rplan".to_string(),
        binary_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: option_env!("GIT_COMMIT").map(str::to_string),
        build_profile: None,
        solver: None,
    };
    let generated_at = args
        .fixed_generated_at
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

    let algorithm_lineage = build_algorithm_lineage(&args)?;
    let certificate = audit_plan_with_lineage(
        &document.plan,
        context.as_ref(),
        &profile,
        runtime,
        &constraints,
        &generated_at,
        algorithm_lineage,
    )?;
    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&certificate)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&certificate)?,
    };

    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }

    match certificate.result {
        AuditResult::Pass => Ok(0),
        AuditResult::PassWithWarnings if args.allow_warnings => Ok(0),
        AuditResult::PassWithWarnings => {
            eprintln!("audit passed with warnings");
            Ok(1)
        }
        AuditResult::Fail => {
            eprintln!("audit failed");
            Ok(1)
        }
    }
}

fn run_verify_certificate(args: VerifyCertificateArgs) -> Result<i32> {
    let cert_text = std::fs::read_to_string(&args.certificate)
        .with_context(|| format!("reading certificate {}", args.certificate.display()))?;
    let certificate = serde_json::from_str::<AuditCertificate>(&cert_text)
        .with_context(|| format!("parsing certificate {}", args.certificate.display()))?;

    let document = if let Some(path) = &args.plan {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading plan {}", path.display()))?;
        Some(
            rplan_io::read_rplan_str(&text)
                .with_context(|| format!("parsing plan {}", path.display()))?,
        )
    } else {
        None
    };

    let context = if let Some(path) = &args.context {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading context {}", path.display()))?;
        Some(
            rplan_io::read_rctx_str(&text)
                .with_context(|| format!("parsing context {}", path.display()))?,
        )
    } else {
        None
    };

    match verify_audit_certificate(
        &certificate,
        document.as_ref().map(|document| &document.plan),
        context.as_ref(),
    ) {
        Ok(verification) => {
            let output = serde_json::json!({
                "verification": "pass",
                "certificate_id": verification.certificate_id,
                "content_hash": verification.content_hash,
                "plan_hash": verification.plan_hash,
                "context_hash": verification.context_hash,
                "result": verification.result,
            });
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string(&output)?),
                OutputFormat::PrettyJson => println!("{}", serde_json::to_string_pretty(&output)?),
            }
            Ok(0)
        }
        Err(err) => {
            eprintln!("certificate verification failed: {err}");
            Ok(1)
        }
    }
}

fn build_algorithm_lineage(args: &AuditArgs) -> Result<Option<AlgorithmLineage>> {
    let any_lineage = args.lineage_producer_crate.is_some()
        || args.lineage_producer_version.is_some()
        || args.lineage_method.is_some()
        || !args.lineage_parent_plan_hash.is_empty()
        || args.lineage_extra_json.is_some();
    if !any_lineage {
        return Ok(None);
    }
    let producer_crate = args
        .lineage_producer_crate
        .as_deref()
        .context("--lineage-producer-crate is required when lineage fields are supplied")?;
    let producer_version = args
        .lineage_producer_version
        .as_deref()
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    let method = args
        .lineage_method
        .as_deref()
        .context("--lineage-method is required when lineage fields are supplied")?;
    let extra = match &args.lineage_extra_json {
        Some(raw) => serde_json::from_str(raw).context("parsing --lineage-extra-json")?,
        None => serde_json::json!({}),
    };
    Ok(Some(AlgorithmLineage::new(
        producer_crate,
        producer_version,
        method,
        args.lineage_parent_plan_hash.clone(),
        extra,
    )?))
}

fn validate_profile_applicability(
    document: &rplan_io::RplanDocument,
    plan_chamber: &Chamber,
    profile: &LegalProfile,
) -> Result<()> {
    if &profile.chamber != plan_chamber {
        anyhow::bail!(
            "legal profile chamber {:?} does not match plan chamber {:?}",
            profile.chamber,
            plan_chamber
        );
    }
    if profile.jurisdiction != "US" && profile.jurisdiction != document.metadata.jurisdiction {
        anyhow::bail!(
            "legal profile jurisdiction '{}' does not match plan jurisdiction '{}'",
            profile.jurisdiction,
            document.metadata.jurisdiction
        );
    }
    if let Some(plan_year) = document.plan.units.year {
        if profile.year != plan_year {
            anyhow::bail!(
                "legal profile year {} does not match plan year {}",
                profile.year,
                plan_year
            );
        }
    }
    Ok(())
}

fn parse_chamber(value: &str) -> Result<Chamber> {
    match value {
        "congressional" => Ok(Chamber::Congressional),
        "state-house" => Ok(Chamber::StateHouse),
        "state-senate" => Ok(Chamber::StateSenate),
        "local" => Ok(Chamber::Local),
        "custom" => Ok(Chamber::Custom("custom".to_string())),
        other => anyhow::bail!("unknown chamber '{other}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_non_congressional_chambers() {
        assert_eq!(parse_chamber("state-house").unwrap(), Chamber::StateHouse);
        assert_eq!(parse_chamber("state-senate").unwrap(), Chamber::StateSenate);
        assert!(parse_chamber("weird").is_err());
    }
}
