use std::collections::HashMap;

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use serde::Serialize;

use crate::{
    AccountSpec, AqamiProjectSpec, Cluster, Diagnostic, EventSpec, FieldSpec, FrameworkErrorSpec,
    InstructionAccountRole, InstructionAccountSpec, InstructionSpec, PackageSpec, PdaSpec,
    ProgramSpec, SeedSpec,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedProjectSpec {
    pub spec_version: String,
    pub package: NormalizedPackage,
    pub cluster: Option<Cluster>,
    pub programs: Vec<NormalizedProgram>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub crate_name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedProgram {
    pub name: String,
    pub docs: Option<String>,
    pub program_id: Option<String>,
    pub rust_crate_name: String,
    pub rust_module_name: String,
    pub accounts: Vec<NormalizedAccount>,
    pub instructions: Vec<NormalizedInstruction>,
    pub pdas: Vec<NormalizedPda>,
    pub events: Vec<NormalizedEvent>,
    pub errors: Vec<NormalizedError>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedAccount {
    pub name: String,
    pub docs: Option<String>,
    pub rust_type_name: String,
    pub rust_module_name: String,
    pub fields: Vec<NormalizedField>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedField {
    pub name: String,
    pub docs: Option<String>,
    pub aqami_type: String,
    pub rust_field_name: String,
    pub rust_type_name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedInstruction {
    pub name: String,
    pub docs: Option<String>,
    pub rust_function_name: String,
    pub rust_module_name: String,
    pub accounts: Vec<NormalizedInstructionAccount>,
    pub args: Vec<NormalizedField>,
    pub emits: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedInstructionAccount {
    pub name: String,
    pub role: InstructionAccountRole,
    pub is_mut: bool,
    pub is_signer: bool,
    pub pda: Option<String>,
    pub docs: Option<String>,
    pub rust_field_name: String,
    pub rust_type_name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedPda {
    pub name: String,
    pub docs: Option<String>,
    pub rust_const_name: String,
    pub seeds: Vec<SeedSpec>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedEvent {
    pub name: String,
    pub docs: Option<String>,
    pub rust_type_name: String,
    pub fields: Vec<NormalizedField>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedError {
    pub name: String,
    pub code: i64,
    pub message: String,
    pub rust_variant_name: String,
}

pub fn normalize_project_spec(
    project: &AqamiProjectSpec,
) -> Result<NormalizedProjectSpec, Vec<Diagnostic>> {
    let diagnostics = normalization_diagnostics(project);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(build_normalized_project(project))
}

pub fn normalization_diagnostics(project: &AqamiProjectSpec) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    push_normalized_identifier_collision_diagnostics(
        &mut diagnostics,
        "$.programs",
        project.programs.iter().map(|program| {
            (
                program.name.as_str(),
                normalize_snake_identifier(&program.name),
                "program module identifier",
            )
        }),
    );

    for (program_index, program) in project.programs.iter().enumerate() {
        let program_location = format!("$.programs[{program_index}]");

        push_normalized_identifier_collision_diagnostics(
            &mut diagnostics,
            &format!("{program_location}.accounts"),
            program.accounts.iter().map(|account| {
                (
                    account.name.as_str(),
                    normalize_upper_camel_identifier(&account.name),
                    "account Rust type identifier",
                )
            }),
        );
        push_normalized_identifier_collision_diagnostics(
            &mut diagnostics,
            &format!("{program_location}.instructions"),
            program.instructions.iter().map(|instruction| {
                (
                    instruction.name.as_str(),
                    normalize_snake_identifier(&instruction.name),
                    "instruction Rust module identifier",
                )
            }),
        );
        push_normalized_identifier_collision_diagnostics(
            &mut diagnostics,
            &format!("{program_location}.events"),
            program.events.iter().map(|event| {
                (
                    event.name.as_str(),
                    normalize_upper_camel_identifier(&event.name),
                    "event Rust type identifier",
                )
            }),
        );
        push_normalized_identifier_collision_diagnostics(
            &mut diagnostics,
            &format!("{program_location}.errors"),
            program.errors.iter().map(|error| {
                (
                    error.name.as_str(),
                    normalize_upper_camel_identifier(&error.name),
                    "error Rust variant identifier",
                )
            }),
        );
        push_normalized_identifier_collision_diagnostics(
            &mut diagnostics,
            &format!("{program_location}.pdas"),
            program.pdas.iter().map(|pda| {
                (
                    pda.name.as_str(),
                    normalize_shouty_identifier(&pda.name),
                    "PDA Rust const identifier",
                )
            }),
        );

        let known_account_types: HashMap<String, String> = program
            .accounts
            .iter()
            .map(|account| {
                (
                    normalize_snake_identifier(&account.name),
                    normalize_upper_camel_identifier(&account.name),
                )
            })
            .collect();

        for (account_index, account) in program.accounts.iter().enumerate() {
            validate_field_collection(
                &mut diagnostics,
                &account.fields,
                &format!("{program_location}.accounts[{account_index}].fields"),
            );
        }

        for (event_index, event) in program.events.iter().enumerate() {
            validate_field_collection(
                &mut diagnostics,
                &event.fields,
                &format!("{program_location}.events[{event_index}].fields"),
            );
        }

        for (instruction_index, instruction) in program.instructions.iter().enumerate() {
            let instruction_location =
                format!("{program_location}.instructions[{instruction_index}]");
            validate_field_collection(
                &mut diagnostics,
                &instruction.args,
                &format!("{instruction_location}.args"),
            );

            push_normalized_identifier_collision_diagnostics(
                &mut diagnostics,
                &format!("{instruction_location}.accounts"),
                instruction.accounts.iter().map(|account| {
                    (
                        account.name.as_str(),
                        normalize_snake_identifier(&account.name),
                        "instruction account Rust field identifier",
                    )
                }),
            );

            let _ = &known_account_types;
        }
    }

    diagnostics
}

fn build_normalized_project(project: &AqamiProjectSpec) -> NormalizedProjectSpec {
    let package = build_normalized_package(&project.package);
    let programs = project
        .programs
        .iter()
        .map(build_normalized_program)
        .collect();

    NormalizedProjectSpec {
        spec_version: project.spec_version.clone(),
        package,
        cluster: project.cluster.clone(),
        programs,
    }
}

fn build_normalized_package(package: &PackageSpec) -> NormalizedPackage {
    NormalizedPackage {
        name: package.name.clone(),
        version: package.version.clone(),
        description: package.description.clone(),
        crate_name: normalize_snake_identifier(&package.name),
    }
}

fn build_normalized_program(program: &ProgramSpec) -> NormalizedProgram {
    let account_type_lookup: HashMap<String, String> = program
        .accounts
        .iter()
        .map(|account| {
            (
                normalize_snake_identifier(&account.name),
                normalize_upper_camel_identifier(&account.name),
            )
        })
        .collect();

    NormalizedProgram {
        name: program.name.clone(),
        docs: program.docs.clone(),
        program_id: program.program_id.clone(),
        rust_crate_name: normalize_snake_identifier(&program.name),
        rust_module_name: normalize_snake_identifier(&program.name),
        accounts: program
            .accounts
            .iter()
            .map(build_normalized_account)
            .collect(),
        instructions: program
            .instructions
            .iter()
            .map(|instruction| build_normalized_instruction(instruction, &account_type_lookup))
            .collect(),
        pdas: program.pdas.iter().map(build_normalized_pda).collect(),
        events: program.events.iter().map(build_normalized_event).collect(),
        errors: program.errors.iter().map(build_normalized_error).collect(),
    }
}

fn build_normalized_account(account: &AccountSpec) -> NormalizedAccount {
    NormalizedAccount {
        name: account.name.clone(),
        docs: account.docs.clone(),
        rust_type_name: normalize_upper_camel_identifier(&account.name),
        rust_module_name: normalize_snake_identifier(&account.name),
        fields: account.fields.iter().map(build_normalized_field).collect(),
    }
}

fn build_normalized_field(field: &FieldSpec) -> NormalizedField {
    NormalizedField {
        name: field.name.clone(),
        docs: field.docs.clone(),
        aqami_type: field.field_type.clone(),
        rust_field_name: normalize_snake_identifier(&field.name),
        rust_type_name: rust_type_name(&field.field_type)
            .expect("normalization should only run after type diagnostics pass")
            .to_string(),
    }
}

fn build_normalized_instruction(
    instruction: &InstructionSpec,
    account_type_lookup: &HashMap<String, String>,
) -> NormalizedInstruction {
    NormalizedInstruction {
        name: instruction.name.clone(),
        docs: instruction.docs.clone(),
        rust_function_name: normalize_snake_identifier(&instruction.name),
        rust_module_name: normalize_snake_identifier(&instruction.name),
        accounts: instruction
            .accounts
            .iter()
            .map(|account| build_normalized_instruction_account(account, account_type_lookup))
            .collect(),
        args: instruction
            .args
            .iter()
            .map(build_normalized_field)
            .collect(),
        emits: instruction.emits.clone(),
        errors: instruction.errors.clone(),
    }
}

fn build_normalized_instruction_account(
    account: &InstructionAccountSpec,
    account_type_lookup: &HashMap<String, String>,
) -> NormalizedInstructionAccount {
    NormalizedInstructionAccount {
        name: account.name.clone(),
        role: account.role.clone(),
        is_mut: account.is_mut,
        is_signer: account.is_signer,
        pda: account.pda.clone(),
        docs: account.docs.clone(),
        rust_field_name: normalize_snake_identifier(&account.name),
        rust_type_name: infer_instruction_account_rust_type(account, account_type_lookup),
    }
}

fn build_normalized_pda(pda: &PdaSpec) -> NormalizedPda {
    NormalizedPda {
        name: pda.name.clone(),
        docs: pda.docs.clone(),
        rust_const_name: normalize_shouty_identifier(&pda.name),
        seeds: pda.seeds.clone(),
    }
}

fn build_normalized_event(event: &EventSpec) -> NormalizedEvent {
    NormalizedEvent {
        name: event.name.clone(),
        docs: None,
        rust_type_name: normalize_upper_camel_identifier(&event.name),
        fields: event.fields.iter().map(build_normalized_field).collect(),
    }
}

fn build_normalized_error(error: &FrameworkErrorSpec) -> NormalizedError {
    NormalizedError {
        name: error.name.clone(),
        code: error.code,
        message: error.message.clone(),
        rust_variant_name: normalize_upper_camel_identifier(&error.name),
    }
}

fn validate_field_collection(
    diagnostics: &mut Vec<Diagnostic>,
    fields: &[FieldSpec],
    location: &str,
) {
    push_normalized_identifier_collision_diagnostics(
        diagnostics,
        location,
        fields.iter().map(|field| {
            (
                field.name.as_str(),
                normalize_snake_identifier(&field.name),
                "Rust field identifier",
            )
        }),
    );

    for (field_index, field) in fields.iter().enumerate() {
        if rust_type_name(&field.field_type).is_none() {
            diagnostics.push(Diagnostic {
                location: format!("{location}[{field_index}]"),
                message: format!(
                    "unsupported AQAMI type `{}` for Rust generation",
                    field.field_type
                ),
            });
        }
    }
}

fn push_normalized_identifier_collision_diagnostics<'a>(
    diagnostics: &mut Vec<Diagnostic>,
    location: &str,
    values: impl Iterator<Item = (&'a str, String, &'static str)>,
) {
    let mut seen: HashMap<String, &str> = HashMap::new();

    for (original_name, normalized_name, label) in values {
        match seen.get(normalized_name.as_str()) {
            Some(existing_name) if *existing_name != original_name => diagnostics.push(Diagnostic {
                location: location.to_string(),
                message: format!(
                    "{label} collision after normalization: `{existing_name}` and `{original_name}` both map to `{normalized_name}`"
                ),
            }),
            None => {
                seen.insert(normalized_name, original_name);
            }
            Some(_) => {}
        }
    }
}

fn infer_instruction_account_rust_type(
    account: &InstructionAccountSpec,
    account_type_lookup: &HashMap<String, String>,
) -> String {
    if matches!(
        account.role,
        InstructionAccountRole::SystemProgram
            | InstructionAccountRole::TokenProgram
            | InstructionAccountRole::Sysvar
            | InstructionAccountRole::Signer
    ) {
        return "Pubkey".to_string();
    }

    let normalized_name = normalize_snake_identifier(&account.name);
    account_type_lookup
        .get(&normalized_name)
        .cloned()
        .unwrap_or_else(|| "Pubkey".to_string())
}

pub fn rust_type_name(aqami_type: &str) -> Option<&'static str> {
    match aqami_type {
        "bool" => Some("bool"),
        "u8" => Some("u8"),
        "u16" => Some("u16"),
        "u32" => Some("u32"),
        "u64" => Some("u64"),
        "u128" => Some("u128"),
        "i8" => Some("i8"),
        "i16" => Some("i16"),
        "i32" => Some("i32"),
        "i64" => Some("i64"),
        "i128" => Some("i128"),
        "string" => Some("String"),
        "bytes" => Some("Vec<u8>"),
        "pubkey" => Some("Pubkey"),
        _ => None,
    }
}

fn normalize_snake_identifier(value: &str) -> String {
    value.to_snake_case()
}

fn normalize_upper_camel_identifier(value: &str) -> String {
    value.to_upper_camel_case()
}

fn normalize_shouty_identifier(value: &str) -> String {
    value.to_shouty_snake_case()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::load_project_spec;

    use super::*;

    fn example_project() -> AqamiProjectSpec {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/specs/escrow.aqami.yaml");
        load_project_spec(path)
            .expect("example should load")
            .project
    }

    #[test]
    fn example_project_normalizes() {
        let project = example_project();
        let normalized = normalize_project_spec(&project).expect("example should normalize");

        assert_eq!(normalized.programs.len(), 1);
        assert_eq!(normalized.programs[0].rust_crate_name, "escrow");
        assert_eq!(normalized.programs[0].accounts[0].rust_type_name, "Escrow");
        assert_eq!(
            normalized.programs[0].instructions[0].accounts[2].rust_type_name,
            "Escrow"
        );
    }

    #[test]
    fn unsupported_types_are_reported() {
        let mut project = example_project();
        project.programs[0].accounts[0].fields[0].field_type = "decimal".to_string();

        let diagnostics = normalization_diagnostics(&project);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("unsupported AQAMI type"))
        );
    }
}
