use std::collections::HashMap;

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use serde::Serialize;

use crate::{
    AccountOwner, AccountSpec, AqamiProjectSpec, Cluster, Diagnostic, EventSpec, FieldSpec,
    FrameworkErrorSpec, HasOneConstraintSpec, InstructionAccountConstraintsSpec,
    InstructionAccountRole, InstructionAccountSpec, InstructionSpec, PackageSpec, PdaBumpKind,
    PdaBumpSpec, PdaSpec, ProgramSpec, SeedSpec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct AccountBinding {
    rust_module_name: String,
    rust_type_name: String,
    owner: NormalizedAccountOwner,
    space: Option<u64>,
}

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
    pub owner: NormalizedAccountOwner,
    pub space: Option<u64>,
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
    pub account_type: Option<String>,
    pub owner: Option<NormalizedAccountOwner>,
    pub space: Option<u64>,
    pub constraints: Option<NormalizedInstructionAccountConstraints>,
    pub is_mut: bool,
    pub is_signer: bool,
    pub pda: Option<String>,
    pub docs: Option<String>,
    pub rust_field_name: String,
    pub rust_type_name: String,
    pub state_account_module_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum NormalizedAccountOwner {
    Program,
    SystemProgram,
    TokenProgram,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedInstructionAccountConstraints {
    pub init: bool,
    pub payer: Option<String>,
    pub close_to: Option<String>,
    pub rent_exempt: bool,
    pub has_one: Vec<NormalizedHasOneConstraint>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedHasOneConstraint {
    pub field: String,
    pub account: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedPda {
    pub name: String,
    pub docs: Option<String>,
    pub rust_const_name: String,
    pub seeds: Vec<SeedSpec>,
    pub bump: Option<NormalizedPdaBump>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NormalizedPdaBump {
    pub kind: NormalizedPdaBumpKind,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum NormalizedPdaBumpKind {
    Canonical,
    Arg,
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

        let declared_account_names: HashMap<&str, AccountBinding> =
            program
                .accounts
                .iter()
                .map(|account| {
                    (
                        account.name.as_str(),
                        AccountBinding {
                            rust_module_name: normalize_snake_identifier(&account.name),
                            rust_type_name: normalize_upper_camel_identifier(&account.name),
                            owner: normalize_account_owner(account.owner.as_ref().expect(
                                "normalization should only run after owner diagnostics pass",
                            )),
                            space: account.space,
                        },
                    )
                })
                .collect();

        for (account_index, account) in program.accounts.iter().enumerate() {
            validate_field_collection(
                &mut diagnostics,
                &account.fields,
                &format!("{program_location}.accounts[{account_index}].fields"),
            );
            if account.owner.is_none() {
                diagnostics.push(Diagnostic {
                    location: format!("{program_location}.accounts[{account_index}].owner"),
                    message: "declared account types must specify `owner`".to_string(),
                });
            }
            if matches!(account.owner, Some(AccountOwner::Program)) && account.space.is_none() {
                diagnostics.push(Diagnostic {
                    location: format!("{program_location}.accounts[{account_index}].space"),
                    message: "program-owned account types should declare `space`".to_string(),
                });
            }
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
            for (account_index, account) in instruction.accounts.iter().enumerate() {
                let account_location = format!("{instruction_location}.accounts[{account_index}]");

                if let Some(account_type) = account.account_type.as_deref() {
                    if account.role != InstructionAccountRole::Account {
                        diagnostics.push(Diagnostic {
                            location: format!("{account_location}.role"),
                            message:
                                "`accountType` is only valid for instruction accounts with role `account`"
                                    .to_string(),
                        });
                    }
                    if !declared_account_names.contains_key(account_type) {
                        diagnostics.push(Diagnostic {
                            location: format!("{account_location}.accountType"),
                            message: format!(
                                "references unknown declared account type `{account_type}`"
                            ),
                        });
                    }
                }

                if account.pda.is_some() && account.account_type.is_none() {
                    diagnostics.push(Diagnostic {
                        location: format!("{account_location}.accountType"),
                        message: "PDA-backed instruction accounts must declare `accountType`"
                            .to_string(),
                    });
                }

                if let Some(constraints) = account.constraints.as_ref() {
                    if account.role != InstructionAccountRole::Account {
                        diagnostics.push(Diagnostic {
                            location: format!("{account_location}.constraints"),
                            message:
                                "instruction account constraints are only valid for role `account`"
                                    .to_string(),
                        });
                    }

                    if constraints.init {
                        if !account.is_mut {
                            diagnostics.push(Diagnostic {
                                location: format!("{account_location}.isMut"),
                                message:
                                    "initialized instruction accounts must set `isMut` to true"
                                        .to_string(),
                            });
                        }
                        if account.account_type.is_none() {
                            diagnostics.push(Diagnostic {
                                location: format!("{account_location}.accountType"),
                                message:
                                    "initialized instruction accounts must declare `accountType`"
                                        .to_string(),
                            });
                        }
                        if constraints.payer.is_none() {
                            diagnostics.push(Diagnostic {
                                location: format!("{account_location}.constraints.payer"),
                                message:
                                    "initialized instruction accounts must declare `constraints.payer`"
                                        .to_string(),
                            });
                        }
                        if let Some(account_type) = account.account_type.as_deref()
                            && let Some(declared_account) = program
                                .accounts
                                .iter()
                                .find(|candidate| candidate.name == account_type)
                            && matches!(declared_account.owner, Some(AccountOwner::Program))
                            && declared_account.space.is_none()
                        {
                            diagnostics.push(Diagnostic {
                                location: format!("{account_location}.accountType"),
                                message: format!(
                                    "initialized program-owned account type `{account_type}` must declare `space`"
                                ),
                            });
                        }
                    }

                    if !constraints.has_one.is_empty() && account.account_type.is_none() {
                        diagnostics.push(Diagnostic {
                            location: format!("{account_location}.accountType"),
                            message:
                                "instruction accounts with `constraints.hasOne` must declare `accountType`"
                                    .to_string(),
                        });
                    }

                    if let Some(account_type) = account.account_type.as_deref()
                        && let Some(declared_account) = program
                            .accounts
                            .iter()
                            .find(|candidate| candidate.name == account_type)
                    {
                        for (has_one_index, has_one) in constraints.has_one.iter().enumerate() {
                            match declared_account
                                .fields
                                .iter()
                                .find(|field| field.name == has_one.field)
                            {
                                Some(field) if field.field_type == "pubkey" => {}
                                Some(field) => diagnostics.push(Diagnostic {
                                    location: format!(
                                        "{account_location}.constraints.hasOne[{has_one_index}].field"
                                    ),
                                    message: format!(
                                        "`constraints.hasOne` field `{}` on account type `{account_type}` must use AQAMI type `pubkey`, found `{}`",
                                        has_one.field, field.field_type
                                    ),
                                }),
                                None => diagnostics.push(Diagnostic {
                                    location: format!(
                                        "{account_location}.constraints.hasOne[{has_one_index}].field"
                                    ),
                                    message: format!(
                                        "`constraints.hasOne` references unknown field `{}` on account type `{account_type}`",
                                        has_one.field
                                    ),
                                }),
                            }

                            if !instruction
                                .accounts
                                .iter()
                                .any(|candidate| candidate.name == has_one.account)
                            {
                                diagnostics.push(Diagnostic {
                                    location: format!(
                                        "{account_location}.constraints.hasOne[{has_one_index}].account"
                                    ),
                                    message: format!(
                                        "`constraints.hasOne` references unknown instruction account `{}`",
                                        has_one.account
                                    ),
                                });
                            }
                        }
                    }

                    if let Some(payer) = constraints.payer.as_deref() {
                        match instruction.accounts.iter().find(|candidate| candidate.name == payer) {
                            Some(payer_account) => {
                                if !payer_account.is_signer {
                                    diagnostics.push(Diagnostic {
                                        location: format!("{account_location}.constraints.payer"),
                                        message: format!(
                                            "payer account `{payer}` must set `isSigner` to true"
                                        ),
                                    });
                                }
                            }
                            None => diagnostics.push(Diagnostic {
                                location: format!("{account_location}.constraints.payer"),
                                message: format!(
                                    "constraints.payer references unknown instruction account `{payer}`"
                                ),
                            }),
                        }
                    }

                    if let Some(close_to) = constraints.close_to.as_deref()
                        && !instruction
                            .accounts
                            .iter()
                            .any(|candidate| candidate.name == close_to)
                    {
                        diagnostics.push(Diagnostic {
                            location: format!("{account_location}.constraints.closeTo"),
                            message: format!(
                                "constraints.closeTo references unknown instruction account `{close_to}`"
                            ),
                        });
                    }
                }
            }
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
    let account_binding_by_name: HashMap<String, AccountBinding> = program
        .accounts
        .iter()
        .map(|account| {
            (
                account.name.clone(),
                AccountBinding {
                    rust_module_name: normalize_snake_identifier(&account.name),
                    rust_type_name: normalize_upper_camel_identifier(&account.name),
                    owner: normalize_account_owner(
                        account
                            .owner
                            .as_ref()
                            .expect("normalization should only run after owner diagnostics pass"),
                    ),
                    space: account.space,
                },
            )
        })
        .collect();
    let account_binding_by_normalized_name: HashMap<String, AccountBinding> = program
        .accounts
        .iter()
        .map(|account| {
            (
                normalize_snake_identifier(&account.name),
                AccountBinding {
                    rust_module_name: normalize_snake_identifier(&account.name),
                    rust_type_name: normalize_upper_camel_identifier(&account.name),
                    owner: normalize_account_owner(
                        account
                            .owner
                            .as_ref()
                            .expect("normalization should only run after owner diagnostics pass"),
                    ),
                    space: account.space,
                },
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
            .map(|instruction| {
                build_normalized_instruction(
                    instruction,
                    &account_binding_by_name,
                    &account_binding_by_normalized_name,
                )
            })
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
        owner: normalize_account_owner(
            account
                .owner
                .as_ref()
                .expect("normalization should only run after owner diagnostics pass"),
        ),
        space: account.space,
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
    account_binding_by_name: &HashMap<String, AccountBinding>,
    account_binding_by_normalized_name: &HashMap<String, AccountBinding>,
) -> NormalizedInstruction {
    NormalizedInstruction {
        name: instruction.name.clone(),
        docs: instruction.docs.clone(),
        rust_function_name: normalize_snake_identifier(&instruction.name),
        rust_module_name: normalize_snake_identifier(&instruction.name),
        accounts: instruction
            .accounts
            .iter()
            .map(|account| {
                build_normalized_instruction_account(
                    account,
                    account_binding_by_name,
                    account_binding_by_normalized_name,
                )
            })
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
    account_binding_by_name: &HashMap<String, AccountBinding>,
    account_binding_by_normalized_name: &HashMap<String, AccountBinding>,
) -> NormalizedInstructionAccount {
    let binding = resolve_instruction_account_binding(
        account,
        account_binding_by_name,
        account_binding_by_normalized_name,
    );
    NormalizedInstructionAccount {
        name: account.name.clone(),
        role: account.role.clone(),
        account_type: account.account_type.clone(),
        owner: binding.as_ref().map(|binding| binding.owner.clone()),
        space: binding.as_ref().and_then(|binding| binding.space),
        constraints: account
            .constraints
            .as_ref()
            .map(build_normalized_instruction_account_constraints),
        is_mut: account.is_mut,
        is_signer: account.is_signer,
        pda: account.pda.clone(),
        docs: account.docs.clone(),
        rust_field_name: normalize_snake_identifier(&account.name),
        rust_type_name: binding
            .as_ref()
            .map(|binding| binding.rust_type_name.clone())
            .unwrap_or_else(|| "Pubkey".to_string()),
        state_account_module_name: binding.map(|binding| binding.rust_module_name),
    }
}

fn build_normalized_instruction_account_constraints(
    constraints: &InstructionAccountConstraintsSpec,
) -> NormalizedInstructionAccountConstraints {
    NormalizedInstructionAccountConstraints {
        init: constraints.init,
        payer: constraints.payer.clone(),
        close_to: constraints.close_to.clone(),
        rent_exempt: constraints.rent_exempt,
        has_one: constraints
            .has_one
            .iter()
            .map(build_normalized_has_one)
            .collect(),
    }
}

fn build_normalized_has_one(constraint: &HasOneConstraintSpec) -> NormalizedHasOneConstraint {
    NormalizedHasOneConstraint {
        field: constraint.field.clone(),
        account: constraint.account.clone(),
    }
}

fn build_normalized_pda(pda: &PdaSpec) -> NormalizedPda {
    NormalizedPda {
        name: pda.name.clone(),
        docs: pda.docs.clone(),
        rust_const_name: normalize_shouty_identifier(&pda.name),
        seeds: pda.seeds.clone(),
        bump: pda.bump.as_ref().map(build_normalized_pda_bump),
    }
}

fn build_normalized_pda_bump(bump: &PdaBumpSpec) -> NormalizedPdaBump {
    NormalizedPdaBump {
        kind: match bump.kind {
            PdaBumpKind::Canonical => NormalizedPdaBumpKind::Canonical,
            PdaBumpKind::Arg => NormalizedPdaBumpKind::Arg,
        },
        value: bump.value.clone(),
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

fn normalize_account_owner(owner: &AccountOwner) -> NormalizedAccountOwner {
    match owner {
        AccountOwner::Program => NormalizedAccountOwner::Program,
        AccountOwner::SystemProgram => NormalizedAccountOwner::SystemProgram,
        AccountOwner::TokenProgram => NormalizedAccountOwner::TokenProgram,
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

fn resolve_instruction_account_binding(
    account: &InstructionAccountSpec,
    account_binding_by_name: &HashMap<String, AccountBinding>,
    account_binding_by_normalized_name: &HashMap<String, AccountBinding>,
) -> Option<AccountBinding> {
    if matches!(
        account.role,
        InstructionAccountRole::SystemProgram
            | InstructionAccountRole::TokenProgram
            | InstructionAccountRole::Sysvar
            | InstructionAccountRole::Signer
    ) {
        return None;
    }

    if let Some(account_type) = account.account_type.as_ref() {
        return account_binding_by_name.get(account_type).cloned();
    }

    let normalized_name = normalize_snake_identifier(&account.name);
    account_binding_by_normalized_name
        .get(&normalized_name)
        .cloned()
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
            normalized.programs[0].accounts[0].owner,
            NormalizedAccountOwner::Program
        );
        assert_eq!(normalized.programs[0].accounts[0].space, Some(128));
        assert_eq!(
            normalized.programs[0].instructions[0].accounts[2].rust_type_name,
            "Escrow"
        );
        assert_eq!(
            normalized.programs[0].instructions[0].accounts[2]
                .state_account_module_name
                .as_deref(),
            Some("escrow")
        );
        assert_eq!(
            normalized.programs[0].instructions[0].accounts[2].space,
            Some(128)
        );
        assert!(
            normalized.programs[0].instructions[0].accounts[2]
                .constraints
                .as_ref()
                .is_some_and(|constraints| constraints.init && constraints.rent_exempt)
        );
        assert_eq!(
            normalized.programs[0].instructions[1].accounts[2]
                .constraints
                .as_ref()
                .map(|constraints| constraints.has_one.len()),
            Some(2)
        );
        assert_eq!(
            normalized.programs[0].pdas[0]
                .bump
                .as_ref()
                .map(|bump| &bump.kind),
            Some(&NormalizedPdaBumpKind::Canonical)
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

    #[test]
    fn pda_backed_instruction_account_requires_account_type() {
        let mut project = example_project();
        project.programs[0].instructions[0].accounts[2].account_type = None;

        let diagnostics = normalization_diagnostics(&project);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("PDA-backed instruction accounts must declare `accountType`")
        }));
    }

    #[test]
    fn initialized_instruction_account_requires_payer() {
        let mut project = example_project();
        let constraints = project.programs[0].instructions[0].accounts[2]
            .constraints
            .as_mut()
            .expect("example should have constraints");
        constraints.payer = None;

        let diagnostics = normalization_diagnostics(&project);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("initialized instruction accounts must declare `constraints.payer`")
        }));
    }

    #[test]
    fn program_owned_account_should_declare_space() {
        let mut project = example_project();
        project.programs[0].accounts[0].space = None;

        let diagnostics = normalization_diagnostics(&project);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("program-owned account types should declare `space`")
        }));
    }

    #[test]
    fn has_one_requires_pubkey_fields() {
        let mut project = example_project();
        project.programs[0].accounts[0].fields[0].field_type = "u64".to_string();

        let diagnostics = normalization_diagnostics(&project);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("must use AQAMI type `pubkey`") })
        );
    }
}
