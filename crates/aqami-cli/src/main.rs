use std::path::{Path, PathBuf};

use aqami_codegen::{GenerateError, generate_rust_programs};
use aqami_spec::{
    Diagnostic, LoadedProjectSpec, ProjectInspection, SpecLoadError, load_project_spec,
    normalize_project_spec, validate_project_spec,
};
use clap::{Parser, Subcommand, ValueEnum};
use thiserror::Error;

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { spec, format } => validate_command(spec, format),
        Command::Inspect { spec, format } => inspect_command(spec, format),
        Command::Generate {
            target,
            spec,
            output_dir,
        } => generate_command(target, spec, output_dir),
    }
}

#[derive(Debug, Error)]
enum CliError {
    #[error("failed to load AQAMI spec from {path}: {source}")]
    LoadSpec {
        path: PathBuf,
        #[source]
        source: SpecLoadError,
    },
    #[error("failed to render {context} as JSON: {source}")]
    RenderJson {
        context: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("AQAMI spec validation failed")]
    InvalidSpec,
    #[error("cannot inspect an invalid AQAMI spec")]
    InspectInvalidSpec,
    #[error("AQAMI normalization failed:\n{diagnostics}")]
    Normalization { diagnostics: String },
    #[error("cannot generate from an invalid AQAMI spec")]
    GenerateInvalidSpec,
    #[error("failed to generate Rust program skeletons under {path}: {source}")]
    GenerateRustProgram {
        path: PathBuf,
        #[source]
        source: GenerateError,
    },
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
    Generate {
        #[arg(value_enum)]
        target: GenerateTarget,
        #[arg(long, value_name = "SPEC_PATH")]
        spec: PathBuf,
        #[arg(long, value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GenerateTarget {
    RustProgram,
}

fn validate_command(spec: PathBuf, format: OutputFormat) -> Result<(), CliError> {
    let loaded = load_spec_with_cli_error(&spec)?;
    let outcome = validate_project_spec(&loaded.project, &loaded.raw_value);

    match format {
        OutputFormat::Text => {
            print_validation_text(&loaded.path, &loaded.project.package.name, &outcome)
        }
        OutputFormat::Json => print_json("validation output", &outcome)?,
    }

    if outcome.is_valid {
        Ok(())
    } else {
        Err(CliError::InvalidSpec)
    }
}

fn inspect_command(spec: PathBuf, format: OutputFormat) -> Result<(), CliError> {
    let loaded = load_spec_with_cli_error(&spec)?;
    let outcome = validate_project_spec(&loaded.project, &loaded.raw_value);

    if !outcome.is_valid {
        match format {
            OutputFormat::Text => {
                print_validation_text(&loaded.path, &loaded.project.package.name, &outcome)
            }
            OutputFormat::Json => print_json("validation output", &outcome)?,
        }
        return Err(CliError::InspectInvalidSpec);
    }

    let inspection = ProjectInspection::from(&loaded.project);

    match format {
        OutputFormat::Text => print_inspection_text(&loaded.path, &inspection),
        OutputFormat::Json => print_json("inspection output", &inspection)?,
    }

    Ok(())
}

fn generate_command(
    target: GenerateTarget,
    spec: PathBuf,
    output_dir: PathBuf,
) -> Result<(), CliError> {
    let loaded = load_spec_with_cli_error(&spec)?;
    let outcome = validate_project_spec(&loaded.project, &loaded.raw_value);
    if !outcome.is_valid {
        print_validation_text(&loaded.path, &loaded.project.package.name, &outcome);
        return Err(CliError::GenerateInvalidSpec);
    }

    let normalized =
        normalize_project_spec(&loaded.project).map_err(|diagnostics| CliError::Normalization {
            diagnostics: format_diagnostics(&diagnostics),
        })?;

    match target {
        GenerateTarget::RustProgram => {
            let generated = generate_rust_programs(&normalized, &output_dir).map_err(|source| {
                CliError::GenerateRustProgram {
                    path: output_dir.clone(),
                    source,
                }
            })?;
            println!("Generated Rust program skeletons:");
            for program in generated {
                println!(
                    "- {} -> {}",
                    program.program_name,
                    program.output_dir.display()
                );
            }
        }
    }

    Ok(())
}

fn load_spec_with_cli_error(path: &Path) -> Result<LoadedProjectSpec, CliError> {
    load_project_spec(path).map_err(|source| CliError::LoadSpec {
        path: path.to_path_buf(),
        source,
    })
}

fn print_json<T: serde::Serialize>(context: &'static str, value: &T) -> Result<(), CliError> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|source| CliError::RenderJson { context, source })?;
    println!("{rendered}");
    Ok(())
}

fn print_validation_text(path: &Path, package_name: &str, outcome: &aqami_spec::ValidationOutcome) {
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

fn print_inspection_text(path: &Path, inspection: &ProjectInspection) {
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

fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str("- ");
        output.push_str(&diagnostic.location);
        output.push_str(": ");
        output.push_str(&diagnostic.message);
    }
    output
}
