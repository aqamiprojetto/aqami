use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::Serialize;

use crate::{
    AqamiProjectSpec, FrameworkErrorSpec, InstructionAccountRole, InstructionSpec,
    PROJECT_SCHEMA_JSON, PdaBumpKind, PdaSpec, SeedKind, SeedSpec, normalization_diagnostics,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Diagnostic {
    pub location: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ValidationOutcome {
    pub is_valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationOutcome {
    fn with_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            is_valid: diagnostics.is_empty(),
            diagnostics,
        }
    }
}

pub fn validate_project_spec(
    project: &AqamiProjectSpec,
    raw_value: &serde_json::Value,
) -> ValidationOutcome {
    let mut diagnostics = schema_diagnostics(raw_value);
    semantic_diagnostics(project, &mut diagnostics);
    diagnostics.extend(normalization_diagnostics(project));
    ValidationOutcome::with_diagnostics(diagnostics)
}

fn schema_diagnostics(raw_value: &serde_json::Value) -> Vec<Diagnostic> {
    static VALIDATOR: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

    let validator = match VALIDATOR.get_or_init(|| {
        let schema_value: serde_json::Value = serde_json::from_str(PROJECT_SCHEMA_JSON)
            .map_err(|error| format!("failed to parse bundled AQAMI project schema: {error}"))?;
        jsonschema::validator_for(&schema_value)
            .map_err(|error| format!("failed to compile bundled AQAMI project schema: {error}"))
    }) {
        Ok(validator) => validator,
        Err(message) => {
            return vec![Diagnostic {
                location: "$schema".to_string(),
                message: message.clone(),
            }];
        }
    };

    validator
        .iter_errors(raw_value)
        .map(|error| Diagnostic {
            location: json_pointer(error.instance_path().as_str()),
            message: error.to_string(),
        })
        .collect()
}

fn semantic_diagnostics(project: &AqamiProjectSpec, diagnostics: &mut Vec<Diagnostic>) {
    push_duplicate_name_diagnostics(
        diagnostics,
        "$.programs",
        project.programs.iter().map(|program| program.name.as_str()),
        "program name",
    );

    for (program_index, program) in project.programs.iter().enumerate() {
        let program_location = format!("$.programs[{program_index}]");
        push_duplicate_name_diagnostics(
            diagnostics,
            &format!("{program_location}.accounts"),
            program.accounts.iter().map(|account| account.name.as_str()),
            "account name",
        );
        push_duplicate_name_diagnostics(
            diagnostics,
            &format!("{program_location}.instructions"),
            program
                .instructions
                .iter()
                .map(|instruction| instruction.name.as_str()),
            "instruction name",
        );
        push_duplicate_name_diagnostics(
            diagnostics,
            &format!("{program_location}.pdas"),
            program.pdas.iter().map(|pda| pda.name.as_str()),
            "PDA name",
        );
        push_duplicate_name_diagnostics(
            diagnostics,
            &format!("{program_location}.events"),
            program.events.iter().map(|event| event.name.as_str()),
            "event name",
        );
        push_duplicate_name_diagnostics(
            diagnostics,
            &format!("{program_location}.errors"),
            program.errors.iter().map(|error| error.name.as_str()),
            "error name",
        );
        push_duplicate_error_code_diagnostics(
            diagnostics,
            &format!("{program_location}.errors"),
            &program.errors,
        );

        let known_pdas: HashMap<&str, &PdaSpec> = program
            .pdas
            .iter()
            .map(|pda| (pda.name.as_str(), pda))
            .collect();
        let known_events: HashSet<&str> = program
            .events
            .iter()
            .map(|event| event.name.as_str())
            .collect();
        let known_errors: HashSet<&str> = program
            .errors
            .iter()
            .map(|error| error.name.as_str())
            .collect();

        for (account_index, account) in program.accounts.iter().enumerate() {
            push_duplicate_name_diagnostics(
                diagnostics,
                &format!("{program_location}.accounts[{account_index}].fields"),
                account.fields.iter().map(|field| field.name.as_str()),
                "account field name",
            );
        }

        for (event_index, event) in program.events.iter().enumerate() {
            push_duplicate_name_diagnostics(
                diagnostics,
                &format!("{program_location}.events[{event_index}].fields"),
                event.fields.iter().map(|field| field.name.as_str()),
                "event field name",
            );
        }

        for (instruction_index, instruction) in program.instructions.iter().enumerate() {
            let instruction_location =
                format!("{program_location}.instructions[{instruction_index}]");
            push_duplicate_name_diagnostics(
                diagnostics,
                &format!("{instruction_location}.accounts"),
                instruction
                    .accounts
                    .iter()
                    .map(|account| account.name.as_str()),
                "instruction account name",
            );
            push_duplicate_name_diagnostics(
                diagnostics,
                &format!("{instruction_location}.args"),
                instruction.args.iter().map(|arg| arg.name.as_str()),
                "instruction argument name",
            );

            validate_instruction_references(
                diagnostics,
                instruction,
                &instruction_location,
                &known_pdas,
                &known_events,
                &known_errors,
            );
        }
    }
}

fn validate_instruction_references(
    diagnostics: &mut Vec<Diagnostic>,
    instruction: &InstructionSpec,
    instruction_location: &str,
    known_pdas: &HashMap<&str, &PdaSpec>,
    known_events: &HashSet<&str>,
    known_errors: &HashSet<&str>,
) {
    let known_instruction_args: HashSet<&str> = instruction
        .args
        .iter()
        .map(|arg| arg.name.as_str())
        .collect();
    let known_instruction_accounts: HashSet<&str> = instruction
        .accounts
        .iter()
        .map(|account| account.name.as_str())
        .collect();

    for (account_index, account) in instruction.accounts.iter().enumerate() {
        let account_location = format!("{instruction_location}.accounts[{account_index}]");

        if let Some(pda_name) = account.pda.as_deref() {
            match known_pdas.get(pda_name) {
                Some(program_pda) => {
                    if account.role != InstructionAccountRole::Account {
                        diagnostics.push(Diagnostic {
                            location: format!("{account_location}.role"),
                            message: "PDA-backed instruction accounts must use the `account` role"
                                .to_string(),
                        });
                    }

                    for (seed_index, seed) in program_pda.seeds.iter().enumerate() {
                        validate_seed_reference(
                            diagnostics,
                            seed,
                            &format!("{account_location}.pda.seeds[{seed_index}]"),
                            &known_instruction_args,
                            &known_instruction_accounts,
                        );
                    }

                    if let Some(bump) = program_pda.bump.as_ref() {
                        validate_pda_bump_reference(
                            diagnostics,
                            bump.kind.clone(),
                            bump.value.as_deref(),
                            &format!("{account_location}.pda.bump"),
                            &known_instruction_args,
                        );
                    }
                }
                None => diagnostics.push(Diagnostic {
                    location: format!("{account_location}.pda"),
                    message: format!("references unknown PDA `{pda_name}`"),
                }),
            }
        }

        match account.role {
            InstructionAccountRole::Signer if !account.is_signer => diagnostics.push(Diagnostic {
                location: format!("{account_location}.isSigner"),
                message: "signer role must set `isSigner` to true".to_string(),
            }),
            InstructionAccountRole::SystemProgram
            | InstructionAccountRole::TokenProgram
            | InstructionAccountRole::Sysvar => {
                if account.is_mut {
                    diagnostics.push(Diagnostic {
                        location: format!("{account_location}.isMut"),
                        message: "program and sysvar accounts cannot be mutable".to_string(),
                    });
                }
                if account.is_signer {
                    diagnostics.push(Diagnostic {
                        location: format!("{account_location}.isSigner"),
                        message: "program and sysvar accounts cannot be signers".to_string(),
                    });
                }
                if account.pda.is_some() {
                    diagnostics.push(Diagnostic {
                        location: format!("{account_location}.pda"),
                        message: "program and sysvar accounts cannot reference PDAs".to_string(),
                    });
                }
            }
            InstructionAccountRole::Account | InstructionAccountRole::Signer => {}
        }
    }

    for (event_index, event_name) in instruction.emits.iter().enumerate() {
        if !known_events.contains(event_name.as_str()) {
            diagnostics.push(Diagnostic {
                location: format!("{instruction_location}.emits[{event_index}]"),
                message: format!("references unknown event `{event_name}`"),
            });
        }
    }

    for (error_index, error_name) in instruction.errors.iter().enumerate() {
        if !known_errors.contains(error_name.as_str()) {
            diagnostics.push(Diagnostic {
                location: format!("{instruction_location}.errors[{error_index}]"),
                message: format!("references unknown error `{error_name}`"),
            });
        }
    }
}

fn validate_pda_bump_reference(
    diagnostics: &mut Vec<Diagnostic>,
    kind: PdaBumpKind,
    value: Option<&str>,
    location: &str,
    known_instruction_args: &HashSet<&str>,
) {
    match kind {
        PdaBumpKind::Canonical => {
            if value.is_some() {
                diagnostics.push(Diagnostic {
                    location: format!("{location}.value"),
                    message: "canonical PDA bumps must not declare `value`".to_string(),
                });
            }
        }
        PdaBumpKind::Arg => {
            let Some(value) = value else {
                diagnostics.push(Diagnostic {
                    location: format!("{location}.value"),
                    message: "arg-backed PDA bumps must declare `value`".to_string(),
                });
                return;
            };

            if !known_instruction_args.contains(value) {
                diagnostics.push(Diagnostic {
                    location: format!("{location}.value"),
                    message: format!("PDA bump references unknown instruction argument `{value}`"),
                });
            }
        }
    }
}

fn validate_seed_reference(
    diagnostics: &mut Vec<Diagnostic>,
    seed: &SeedSpec,
    location: &str,
    known_instruction_args: &HashSet<&str>,
    known_instruction_accounts: &HashSet<&str>,
) {
    match seed.kind {
        SeedKind::Const => {}
        SeedKind::Arg => {
            if !known_instruction_args.contains(seed.value.as_str()) {
                diagnostics.push(Diagnostic {
                    location: location.to_string(),
                    message: format!(
                        "PDA seed references unknown instruction argument `{}`",
                        seed.value
                    ),
                });
            }
        }
        SeedKind::AccountKey => {
            if !known_instruction_accounts.contains(seed.value.as_str()) {
                diagnostics.push(Diagnostic {
                    location: location.to_string(),
                    message: format!(
                        "PDA seed references unknown instruction account `{}`",
                        seed.value
                    ),
                });
            }
        }
        SeedKind::AccountField => {
            let Some((account_name, _field_name)) = seed.value.split_once('.') else {
                diagnostics.push(Diagnostic {
                    location: location.to_string(),
                    message: "account_field seed values must use `account.field`".to_string(),
                });
                return;
            };

            if !known_instruction_accounts.contains(account_name) {
                diagnostics.push(Diagnostic {
                    location: location.to_string(),
                    message: format!(
                        "PDA seed references unknown instruction account `{account_name}`"
                    ),
                });
            }
        }
    }
}

fn push_duplicate_name_diagnostics<'a>(
    diagnostics: &mut Vec<Diagnostic>,
    location: &str,
    values: impl Iterator<Item = &'a str>,
    label: &str,
) {
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();

    for value in values {
        if !seen.insert(value) {
            duplicates.insert(value.to_string());
        }
    }

    for duplicate in duplicates {
        diagnostics.push(Diagnostic {
            location: location.to_string(),
            message: format!("duplicate {label} `{duplicate}`"),
        });
    }
}

fn push_duplicate_error_code_diagnostics(
    diagnostics: &mut Vec<Diagnostic>,
    location: &str,
    errors: &[FrameworkErrorSpec],
) {
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();

    for error in errors {
        if !seen.insert(error.code) {
            duplicates.insert(error.code);
        }
    }

    for duplicate in duplicates {
        diagnostics.push(Diagnostic {
            location: location.to_string(),
            message: format!("duplicate error code `{duplicate}`"),
        });
    }
}

fn json_pointer(instance_path: &str) -> String {
    if instance_path.is_empty() {
        "$".to_string()
    } else {
        format!("${instance_path}")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{load_project_spec, normalize_project_spec};

    use super::*;

    fn parse_example() -> (AqamiProjectSpec, serde_json::Value) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/specs/escrow.aqami.yaml");
        let loaded = load_project_spec(path).expect("example spec should load");
        (loaded.project, loaded.raw_value)
    }

    #[test]
    fn example_spec_is_valid() {
        let (project, raw_value) = parse_example();
        let outcome = validate_project_spec(&project, &raw_value);

        assert!(outcome.is_valid, "diagnostics: {:?}", outcome.diagnostics);
    }

    #[test]
    fn duplicate_program_names_are_reported() {
        let (mut project, _) = parse_example();
        project.programs.push(project.programs[0].clone());
        let raw_value = serde_json::to_value(&project).expect("project should serialize");

        let outcome = validate_project_spec(&project, &raw_value);

        assert!(!outcome.is_valid);
        assert!(
            outcome
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("duplicate program name"))
        );
    }

    #[test]
    fn signer_role_requires_is_signer_flag() {
        let (mut project, _) = parse_example();
        project.programs[0].instructions[0].accounts[0].is_signer = false;
        let raw_value = serde_json::to_value(&project).expect("project should serialize");

        let outcome = validate_project_spec(&project, &raw_value);

        assert!(!outcome.is_valid);
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("signer role must set `isSigner`")
        }));
    }

    #[test]
    fn normalized_example_spec_is_valid() {
        let (project, _) = parse_example();
        let normalized = normalize_project_spec(&project).expect("example should normalize");

        assert_eq!(normalized.programs[0].instructions.len(), 2);
    }

    #[test]
    fn arg_bump_requires_known_instruction_argument() {
        let (mut project, _) = parse_example();
        project.programs[0].pdas[0].bump = Some(crate::PdaBumpSpec {
            kind: crate::PdaBumpKind::Arg,
            value: Some("missing_bump".to_string()),
        });
        let raw_value = serde_json::to_value(&project).expect("project should serialize");

        let outcome = validate_project_spec(&project, &raw_value);

        assert!(!outcome.is_valid);
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("PDA bump references unknown instruction argument `missing_bump`")
        }));
    }

    #[test]
    fn has_one_references_must_exist() {
        let (mut project, _) = parse_example();
        project.programs[0].instructions[1].accounts[2]
            .constraints
            .as_mut()
            .expect("release_escrow should have constraints")
            .has_one[0]
            .account = "missing_authority".to_string();
        let raw_value = serde_json::to_value(&project).expect("project should serialize");

        let outcome = validate_project_spec(&project, &raw_value);

        assert!(!outcome.is_valid);
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("references unknown instruction account `missing_authority`")
        }));
    }
}
