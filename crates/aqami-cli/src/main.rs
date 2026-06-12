use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use aqami_spec::{ProjectInspection, load_project_spec, validate_project_spec};
use clap::{Parser, Subcommand, ValueEnum};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { spec, format } => validate_command(spec, format),
        Command::Inspect { spec, format } => inspect_command(spec, format),
    }
}

#[derive(Debug, Parser)]
#[command(name = "aqami", version, about = "AQAMI CLI foundation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Validate {
        #[arg(value_name = "SPEC_PATH")]
        spec: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    Inspect {
        #[arg(value_name = "SPEC_PATH")]
        spec: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn validate_command(spec: PathBuf, format: OutputFormat) -> Result<()> {
    let loaded = load_project_spec(&spec)
        .with_context(|| format!("failed to load AQAMI spec from {}", spec.display()))?;
    let outcome = validate_project_spec(&loaded.project, &loaded.raw_value);

    match format {
        OutputFormat::Text => {
            print_validation_text(&loaded.path, &loaded.project.package.name, &outcome)
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&outcome)
                    .context("failed to render validation output as JSON")?
            );
        }
    }

    if outcome.is_valid {
        Ok(())
    } else {
        bail!("AQAMI spec validation failed")
    }
}

fn inspect_command(spec: PathBuf, format: OutputFormat) -> Result<()> {
    let loaded = load_project_spec(&spec)
        .with_context(|| format!("failed to load AQAMI spec from {}", spec.display()))?;
    let outcome = validate_project_spec(&loaded.project, &loaded.raw_value);

    if !outcome.is_valid {
        match format {
            OutputFormat::Text => {
                print_validation_text(&loaded.path, &loaded.project.package.name, &outcome)
            }
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&outcome)
                        .context("failed to render validation output as JSON")?
                );
            }
        }
        bail!("cannot inspect an invalid AQAMI spec");
    }

    let inspection = ProjectInspection::from(&loaded.project);

    match format {
        OutputFormat::Text => print_inspection_text(&loaded.path, &inspection),
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&inspection)
                    .context("failed to render inspection output as JSON")?
            );
        }
    }

    Ok(())
}

fn print_validation_text(
    path: &std::path::Path,
    package_name: &str,
    outcome: &aqami_spec::ValidationOutcome,
) {
    if outcome.is_valid {
        println!(
            "AQAMI spec is valid.\npath: {}\npackage: {}",
            path.display(),
            package_name
        );
        return;
    }

    println!(
        "AQAMI spec is invalid.\npath: {}\npackage: {}\ndiagnostics:",
        path.display(),
        package_name
    );

    for diagnostic in &outcome.diagnostics {
        println!("- {}: {}", diagnostic.location, diagnostic.message);
    }
}

fn print_inspection_text(path: &std::path::Path, inspection: &ProjectInspection) {
    println!("AQAMI project inspection");
    println!("path: {}", path.display());
    println!("spec version: {}", inspection.spec_version);
    println!(
        "package: {}@{}",
        inspection.package_name, inspection.package_version
    );

    if let Some(cluster) = &inspection.cluster {
        println!("cluster: {cluster}");
    }

    println!("programs: {}", inspection.program_count);

    for program in &inspection.programs {
        println!();
        println!("program: {}", program.name);
        println!(
            "  accounts: {} | instructions: {} | pdas: {} | events: {} | errors: {}",
            program.account_count,
            program.instruction_count,
            program.pda_count,
            program.event_count,
            program.error_count
        );

        if !program.accounts.is_empty() {
            println!(
                "  account names: {}",
                comma_join(program.accounts.iter().map(|item| item.name.as_str()))
            );
        }
        if !program.instructions.is_empty() {
            println!(
                "  instruction names: {}",
                comma_join(program.instructions.iter().map(|item| item.name.as_str()))
            );
        }
        if !program.pdas.is_empty() {
            println!(
                "  PDA names: {}",
                comma_join(program.pdas.iter().map(|item| item.name.as_str()))
            );
        }
        if !program.events.is_empty() {
            println!(
                "  event names: {}",
                comma_join(program.events.iter().map(|item| item.name.as_str()))
            );
        }
        if !program.errors.is_empty() {
            println!(
                "  error names: {}",
                comma_join(program.errors.iter().map(|item| item.name.as_str()))
            );
        }
    }
}

fn comma_join<'a>(values: impl Iterator<Item = &'a str>) -> String {
    values.collect::<Vec<_>>().join(", ")
}
