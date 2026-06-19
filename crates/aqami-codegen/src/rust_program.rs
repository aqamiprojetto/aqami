use std::{
    collections::BTreeSet,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use aqami_spec::{
    NormalizedAccount, NormalizedAccountOwner, NormalizedError, NormalizedEvent,
    NormalizedInstruction, NormalizedInstructionAccount, NormalizedPda, NormalizedPdaBumpKind,
    NormalizedProgram, NormalizedProjectSpec, SeedKind,
};
use heck::ToUpperCamelCase;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedProgram {
    pub program_name: String,
    pub output_dir: PathBuf,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateRustProgramOptions {
    pub aqami_runtime_path: PathBuf,
}

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("output directory already exists for generated program `{program_name}`: {path}")]
    OutputAlreadyExists { program_name: String, path: PathBuf },
    #[error("failed to create directory {path}: {message}")]
    CreateDirectory { path: PathBuf, message: String },
    #[error("failed to write generated file {path}: {message}")]
    WriteFile { path: PathBuf, message: String },
}

pub fn generate_rust_programs(
    project: &NormalizedProjectSpec,
    output_root: impl AsRef<Path>,
    options: &GenerateRustProgramOptions,
) -> Result<Vec<GeneratedProgram>, GenerateError> {
    let output_root = output_root.as_ref();
    let mut generated_programs = Vec::with_capacity(project.programs.len());

    for program in &project.programs {
        generated_programs.push(generate_rust_program(
            output_root,
            project,
            program,
            options,
        )?);
    }

    Ok(generated_programs)
}

fn generate_rust_program(
    output_root: &Path,
    project: &NormalizedProjectSpec,
    program: &NormalizedProgram,
    options: &GenerateRustProgramOptions,
) -> Result<GeneratedProgram, GenerateError> {
    let program_dir = output_root.join(&program.rust_crate_name);
    if program_dir.exists() {
        return Err(GenerateError::OutputAlreadyExists {
            program_name: program.name.clone(),
            path: program_dir,
        });
    }

    let src_dir = program_dir.join("src");
    let state_dir = src_dir.join("state");
    let instructions_dir = src_dir.join("instructions");

    for dir in [&program_dir, &src_dir, &state_dir, &instructions_dir] {
        fs::create_dir_all(dir).map_err(|error| GenerateError::CreateDirectory {
            path: dir.to_path_buf(),
            message: error.to_string(),
        })?;
    }

    let mut files = Vec::new();
    write_file(
        &program_dir.join("Cargo.toml"),
        &render_cargo_toml(project, program, options),
        &mut files,
    )?;
    write_file(
        &program_dir.join("README.md"),
        &render_program_readme(project, program),
        &mut files,
    )?;
    write_file(
        &src_dir.join("lib.rs"),
        &render_lib_rs(project, program),
        &mut files,
    )?;
    write_file(
        &src_dir.join("errors.rs"),
        &render_errors_rs(&program.errors),
        &mut files,
    )?;
    write_file(
        &src_dir.join("events.rs"),
        &render_events_rs(&program.events),
        &mut files,
    )?;
    write_file(
        &src_dir.join("pdas.rs"),
        &render_pdas_rs(&program.pdas),
        &mut files,
    )?;
    write_file(
        &state_dir.join("mod.rs"),
        &render_state_mod_rs(program),
        &mut files,
    )?;
    write_file(
        &instructions_dir.join("mod.rs"),
        &render_instructions_mod_rs(program),
        &mut files,
    )?;

    for account in &program.accounts {
        write_file(
            &state_dir.join(format!("{}.rs", account.rust_module_name)),
            &render_state_account_rs(account),
            &mut files,
        )?;
    }

    for instruction in &program.instructions {
        write_file(
            &instructions_dir.join(format!("{}.rs", instruction.rust_module_name)),
            &render_instruction_rs(instruction, &program.pdas),
            &mut files,
        )?;
    }

    Ok(GeneratedProgram {
        program_name: program.name.clone(),
        output_dir: program_dir,
        files,
    })
}

fn write_file(path: &Path, contents: &str, files: &mut Vec<PathBuf>) -> Result<(), GenerateError> {
    fs::write(path, contents).map_err(|error| GenerateError::WriteFile {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    files.push(path.to_path_buf());
    Ok(())
}

fn render_cargo_toml(
    project: &NormalizedProjectSpec,
    program: &NormalizedProgram,
    options: &GenerateRustProgramOptions,
) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"lib\"]\n\n[dependencies]\naqami-runtime = {{ path = \"{}\" }}\n",
        program.rust_crate_name,
        project.package.version,
        escape_toml_basic_string(&options.aqami_runtime_path.to_string_lossy()),
    )
}

fn render_program_readme(project: &NormalizedProjectSpec, program: &NormalizedProgram) -> String {
    let mut output = String::new();
    writeln!(&mut output, "# {}", program.name).expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    if let Some(docs) = &program.docs {
        writeln!(&mut output, "{docs}").expect("string write should succeed");
        writeln!(&mut output).expect("string write should succeed");
    }
    writeln!(
        &mut output,
        "Generated by AQAMI from spec version `{}`.",
        project.spec_version
    )
    .expect("string write should succeed");
    output
}

fn render_lib_rs(project: &NormalizedProjectSpec, program: &NormalizedProgram) -> String {
    let mut output = String::new();
    writeln!(&mut output, "pub mod errors;").expect("string write should succeed");
    writeln!(&mut output, "pub mod events;").expect("string write should succeed");
    writeln!(&mut output, "pub mod instructions;").expect("string write should succeed");
    writeln!(&mut output, "pub mod pdas;").expect("string write should succeed");
    writeln!(&mut output, "pub mod state;").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(
        &mut output,
        "pub const AQAMI_SPEC_VERSION: &str = \"{}\";",
        project.spec_version
    )
    .expect("string write should succeed");
    writeln!(
        &mut output,
        "pub const AQAMI_PROGRAM_NAME: &str = \"{}\";",
        program.name
    )
    .expect("string write should succeed");
    output
}

fn render_errors_rs(errors: &[NormalizedError]) -> String {
    let mut output = String::new();
    writeln!(&mut output, "#[derive(Debug, Clone, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(&mut output, "#[repr(i64)]").expect("string write should succeed");
    writeln!(&mut output, "pub enum ProgramError {{").expect("string write should succeed");
    if errors.is_empty() {
        writeln!(&mut output, "    Unknown = 0,").expect("string write should succeed");
    } else {
        for error in errors {
            push_doc_comment(&mut output, 4, Some(&error.message));
            writeln!(
                &mut output,
                "    {} = {},",
                error.rust_variant_name, error.code
            )
            .expect("string write should succeed");
        }
    }
    writeln!(&mut output, "}}").expect("string write should succeed");
    output
}

fn render_events_rs(events: &[NormalizedEvent]) -> String {
    let mut output = String::new();
    if events.is_empty() {
        writeln!(&mut output, "// No events declared in the AQAMI spec.")
            .expect("string write should succeed");
        return output;
    }

    writeln!(&mut output, "use aqami_runtime::Pubkey;").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");

    for event in events {
        push_doc_comment(&mut output, 0, event.docs.as_deref());
        writeln!(&mut output, "#[derive(Debug, Clone, PartialEq, Eq)]")
            .expect("string write should succeed");
        writeln!(&mut output, "pub struct {} {{", event.rust_type_name)
            .expect("string write should succeed");
        for field in &event.fields {
            push_doc_comment(&mut output, 4, field.docs.as_deref());
            writeln!(
                &mut output,
                "    pub {}: {},",
                field.rust_field_name, field.rust_type_name
            )
            .expect("string write should succeed");
        }
        writeln!(&mut output, "}}").expect("string write should succeed");
        writeln!(&mut output).expect("string write should succeed");
    }

    output
}

fn render_pdas_rs(pdas: &[NormalizedPda]) -> String {
    let mut output = String::new();
    if pdas.is_empty() {
        writeln!(&mut output, "// No PDAs declared in the AQAMI spec.")
            .expect("string write should succeed");
        return output;
    }

    writeln!(
        &mut output,
        "use aqami_runtime::{{PdaBumpDescriptor, PdaBumpKindDescriptor, PdaDescriptor, PdaSeedDescriptor, PdaSeedKindDescriptor}};"
    )
    .expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");

    for pda in pdas {
        push_doc_comment(&mut output, 0, pda.docs.as_deref());
        writeln!(
            &mut output,
            "pub const {}: &str = \"{}\";",
            pda.rust_const_name, pda.name
        )
        .expect("string write should succeed");
        writeln!(
            &mut output,
            "pub const {}_DESCRIPTOR: PdaDescriptor = PdaDescriptor {{ name: \"{}\", seeds: {}, bump: {} }};",
            pda.rust_const_name,
            pda.name,
            render_pda_seeds_literal(&pda.seeds),
            render_pda_bump_literal(pda),
        )
        .expect("string write should succeed");
        writeln!(&mut output).expect("string write should succeed");
        writeln!(
            &mut output,
            "pub fn {}_seed_descriptions() -> &'static [&'static str] {{",
            pda.rust_const_name.to_ascii_lowercase()
        )
        .expect("string write should succeed");
        writeln!(&mut output, "    &[").expect("string write should succeed");
        for seed in &pda.seeds {
            writeln!(
                &mut output,
                "        \"{}: {}\",",
                seed_kind_name(&seed.kind),
                seed.value
            )
            .expect("string write should succeed");
        }
        writeln!(&mut output, "    ]").expect("string write should succeed");
        writeln!(&mut output, "}}").expect("string write should succeed");
        writeln!(
            &mut output,
            "pub fn {}_bump_description() -> Option<&'static str> {{",
            pda.rust_const_name.to_ascii_lowercase()
        )
        .expect("string write should succeed");
        writeln!(
            &mut output,
            "    {}",
            render_pda_bump_description_literal(pda)
        )
        .expect("string write should succeed");
        writeln!(&mut output, "}}").expect("string write should succeed");
        writeln!(&mut output).expect("string write should succeed");
    }

    output
}

fn render_state_mod_rs(program: &NormalizedProgram) -> String {
    let mut output = String::new();
    for account in &program.accounts {
        writeln!(&mut output, "pub mod {};", account.rust_module_name)
            .expect("string write should succeed");
    }
    output
}

fn render_instructions_mod_rs(program: &NormalizedProgram) -> String {
    let mut output = String::new();
    for instruction in &program.instructions {
        writeln!(&mut output, "pub mod {};", instruction.rust_module_name)
            .expect("string write should succeed");
    }
    output
}

fn render_state_account_rs(account: &NormalizedAccount) -> String {
    let mut output = String::new();
    writeln!(
        &mut output,
        "use aqami_runtime::{{AccountOwner, AccountTypeDescriptor, Pubkey}};"
    )
    .expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(
        &mut output,
        "pub const ACCOUNT_TYPE_DESCRIPTOR: AccountTypeDescriptor = AccountTypeDescriptor {{ name: \"{}\", owner: AccountOwner::{}, space: {} }};",
        account.name,
        account_owner_variant_name(&account.owner),
        option_u64_literal(account.space),
    )
    .expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    push_doc_comment(&mut output, 0, account.docs.as_deref());
    writeln!(&mut output, "#[derive(Debug, Clone, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(&mut output, "pub struct {} {{", account.rust_type_name)
        .expect("string write should succeed");
    for field in &account.fields {
        push_doc_comment(&mut output, 4, field.docs.as_deref());
        writeln!(
            &mut output,
            "    pub {}: {},",
            field.rust_field_name, field.rust_type_name
        )
        .expect("string write should succeed");
    }
    writeln!(&mut output, "}}").expect("string write should succeed");
    output
}

fn render_instruction_rs(
    instruction: &NormalizedInstruction,
    program_pdas: &[NormalizedPda],
) -> String {
    let mut output = String::new();
    writeln!(&mut output, "use crate::errors::ProgramError;").expect("string write should succeed");
    let mut runtime_imports = vec![
        "AccountInfo",
        "InstructionAccountDescriptor",
        "InstructionAccountRoleDescriptor",
        "Pubkey",
        "RuntimeValidationError",
        "SolanaPubkey",
        "validate_instruction_accounts",
    ];
    if instruction
        .accounts
        .iter()
        .any(|account| account.owner.is_some())
    {
        runtime_imports.push("AccountOwner");
    }
    if instruction
        .accounts
        .iter()
        .any(|account| account.constraints.is_some())
    {
        runtime_imports.push("InstructionAccountConstraintDescriptor");
    }
    if instruction.accounts.iter().any(|account| {
        account
            .constraints
            .as_ref()
            .is_some_and(|constraints| !constraints.has_one.is_empty())
    }) {
        runtime_imports.push("HasOneConstraintDescriptor");
    }
    let referenced_pdas = referenced_pdas(instruction, program_pdas);
    if !referenced_pdas.is_empty() {
        runtime_imports.push("PdaDescriptor");
        runtime_imports.push("validate_program_account_infos_with_pdas");
    } else {
        runtime_imports.push("validate_program_account_infos");
    }
    writeln!(
        &mut output,
        "use aqami_runtime::{{{}}};",
        runtime_imports.join(", ")
    )
    .expect("string write should succeed");

    let account_imports = instruction
        .accounts
        .iter()
        .filter_map(|account| {
            account
                .state_account_module_name
                .as_ref()
                .map(|module_name| format!("state::{module_name}::{}", account.rust_type_name))
        })
        .collect::<BTreeSet<_>>();
    for import in account_imports {
        writeln!(&mut output, "use crate::{import};").expect("string write should succeed");
    }
    if !referenced_pdas.is_empty() {
        writeln!(
            &mut output,
            "use crate::pdas::{{{}}};",
            referenced_pdas
                .iter()
                .map(|pda| format!("{}_DESCRIPTOR", pda.rust_const_name))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .expect("string write should succeed");
    }

    writeln!(&mut output).expect("string write should succeed");
    writeln!(
        &mut output,
        "pub const ACCOUNT_DESCRIPTORS: &[InstructionAccountDescriptor] = &["
    )
    .expect("string write should succeed");
    for account in &instruction.accounts {
        writeln!(
            &mut output,
            "    InstructionAccountDescriptor {{ name: \"{}\", role: InstructionAccountRoleDescriptor::{}, account_type: {}, owner: {}, space: {}, is_mut: {}, is_signer: {}, pda: {}, constraints: {} }},",
            account.name,
            instruction_account_role_variant_name(&account.role),
            option_str_literal(account.account_type.as_deref()),
            option_account_owner_literal(account),
            option_u64_literal(account.space),
            account.is_mut,
            account.is_signer,
            option_str_literal(account.pda.as_deref()),
            render_constraint_literal(account),
        )
        .expect("string write should succeed");
    }
    writeln!(&mut output, "];").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    if !referenced_pdas.is_empty() {
        writeln!(
            &mut output,
            "/// PDA descriptors referenced by this instruction's runtime validation path."
        )
        .expect("string write should succeed");
        writeln!(
            &mut output,
            "pub const PDA_DESCRIPTORS: &[PdaDescriptor] = &[{}];",
            referenced_pdas
                .iter()
                .map(|pda| format!("{}_DESCRIPTOR", pda.rust_const_name))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .expect("string write should succeed");
        writeln!(&mut output).expect("string write should succeed");
    }
    writeln!(
        &mut output,
        "/// Validates real Solana runtime accounts for this instruction against AQAMI descriptors."
    )
    .expect("string write should succeed");
    writeln!(
        &mut output,
        "pub fn validate_runtime_accounts(program_id: &SolanaPubkey, account_infos: &[AccountInfo<'_>]) -> Result<(), RuntimeValidationError> {{"
    )
    .expect("string write should succeed");
    if referenced_pdas.is_empty() {
        writeln!(
            &mut output,
            "    validate_program_account_infos(program_id, ACCOUNT_DESCRIPTORS, account_infos)"
        )
        .expect("string write should succeed");
    } else {
        writeln!(
            &mut output,
            "    validate_program_account_infos_with_pdas(program_id, ACCOUNT_DESCRIPTORS, account_infos, PDA_DESCRIPTORS)"
        )
        .expect("string write should succeed");
    }
    writeln!(&mut output, "}}").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(
        &mut output,
        "/// Validates descriptor-to-descriptor AQAMI invariants for this instruction."
    )
    .expect("string write should succeed");
    writeln!(
        &mut output,
        "pub fn validate_account_descriptors() -> Result<(), RuntimeValidationError> {{"
    )
    .expect("string write should succeed");
    writeln!(
        &mut output,
        "    validate_instruction_accounts(ACCOUNT_DESCRIPTORS)"
    )
    .expect("string write should succeed");
    writeln!(&mut output, "}}").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    push_doc_comment(&mut output, 0, instruction.docs.as_deref());
    writeln!(&mut output, "#[derive(Debug, Clone, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(
        &mut output,
        "pub struct {}Accounts {{",
        instruction_name_prefix(instruction)
    )
    .expect("string write should succeed");
    for account in &instruction.accounts {
        push_doc_comment(&mut output, 4, account.docs.as_deref());
        push_instruction_account_metadata_comment(&mut output, 4, account);
        writeln!(
            &mut output,
            "    pub {}: {},",
            account.rust_field_name, account.rust_type_name
        )
        .expect("string write should succeed");
    }
    writeln!(&mut output, "}}").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(&mut output, "#[derive(Debug, Clone, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(
        &mut output,
        "pub struct {}Args {{",
        instruction_name_prefix(instruction)
    )
    .expect("string write should succeed");
    for field in &instruction.args {
        push_doc_comment(&mut output, 4, field.docs.as_deref());
        writeln!(
            &mut output,
            "    pub {}: {},",
            field.rust_field_name, field.rust_type_name
        )
        .expect("string write should succeed");
    }
    writeln!(&mut output, "}}").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(
        &mut output,
        "pub fn execute(_accounts: &mut {}Accounts, _args: {}Args) -> Result<(), ProgramError> {{",
        instruction_name_prefix(instruction),
        instruction_name_prefix(instruction)
    )
    .expect("string write should succeed");
    writeln!(&mut output, "    todo!(\"Implement {}\")", instruction.name)
        .expect("string write should succeed");
    writeln!(&mut output, "}}").expect("string write should succeed");
    output
}

fn referenced_pdas<'a>(
    instruction: &NormalizedInstruction,
    program_pdas: &'a [NormalizedPda],
) -> Vec<&'a NormalizedPda> {
    let referenced_names = instruction
        .accounts
        .iter()
        .filter_map(|account| account.pda.as_deref())
        .collect::<BTreeSet<_>>();

    program_pdas
        .iter()
        .filter(|pda| referenced_names.contains(pda.name.as_str()))
        .collect()
}

fn push_doc_comment(output: &mut String, indent: usize, docs: Option<&str>) {
    let Some(docs) = docs else {
        return;
    };

    let prefix = " ".repeat(indent);
    for line in docs.lines() {
        writeln!(output, "{prefix}/// {line}").expect("string write should succeed");
    }
}

fn push_instruction_account_metadata_comment(
    output: &mut String,
    indent: usize,
    account: &NormalizedInstructionAccount,
) {
    let prefix = " ".repeat(indent);
    let mut parts = Vec::with_capacity(4);
    parts.push(format!(
        "role={}",
        instruction_account_role_name(&account.role)
    ));
    if let Some(account_type) = account.account_type.as_deref() {
        parts.push(format!("account_type={account_type}"));
    }
    if let Some(owner) = account.owner.as_ref() {
        parts.push(format!("owner={}", owner_literal_name(owner)));
    }
    if let Some(space) = account.space {
        parts.push(format!("space={space}"));
    }
    if account.is_mut {
        parts.push("mut".to_string());
    }
    if account.is_signer {
        parts.push("signer".to_string());
    }
    if let Some(pda) = account.pda.as_deref() {
        parts.push(format!("pda={pda}"));
    }
    if let Some(constraints) = account.constraints.as_ref() {
        if constraints.init {
            parts.push("init".to_string());
        }
        if let Some(payer) = constraints.payer.as_deref() {
            parts.push(format!("payer={payer}"));
        }
        if let Some(close_to) = constraints.close_to.as_deref() {
            parts.push(format!("close_to={close_to}"));
        }
        if constraints.rent_exempt {
            parts.push("rent_exempt".to_string());
        }
        for relation in &constraints.has_one {
            parts.push(format!("has_one={}->{}", relation.field, relation.account));
        }
    }
    writeln!(output, "{prefix}/// {}", parts.join(", ")).expect("string write should succeed");
}

fn instruction_name_prefix(instruction: &NormalizedInstruction) -> String {
    instruction.rust_module_name.to_upper_camel_case()
}

fn seed_kind_name(kind: &SeedKind) -> &'static str {
    match kind {
        SeedKind::Const => "const",
        SeedKind::Arg => "arg",
        SeedKind::AccountField => "account_field",
        SeedKind::AccountKey => "account_key",
    }
}

fn seed_kind_variant_name(kind: &SeedKind) -> &'static str {
    match kind {
        SeedKind::Const => "Const",
        SeedKind::Arg => "Arg",
        SeedKind::AccountField => "AccountField",
        SeedKind::AccountKey => "AccountKey",
    }
}

fn render_pda_seeds_literal(seeds: &[aqami_spec::SeedSpec]) -> String {
    if seeds.is_empty() {
        return "&[]".to_string();
    }

    let members = seeds
        .iter()
        .map(|seed| {
            format!(
                "PdaSeedDescriptor {{ kind: PdaSeedKindDescriptor::{}, value: \"{}\" }}",
                seed_kind_variant_name(&seed.kind),
                seed.value
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("&[{members}]")
}

fn render_pda_bump_literal(pda: &NormalizedPda) -> String {
    match pda.bump.as_ref() {
        Some(bump) => format!(
            "Some(PdaBumpDescriptor {{ kind: PdaBumpKindDescriptor::{}, value: {} }})",
            pda_bump_kind_variant_name(&bump.kind),
            option_str_literal(bump.value.as_deref())
        ),
        None => "None".to_string(),
    }
}

fn render_pda_bump_description_literal(pda: &NormalizedPda) -> String {
    match pda.bump.as_ref() {
        Some(bump) => match bump.value.as_deref() {
            Some(value) => format!("Some(\"{}: {}\")", pda_bump_kind_name(&bump.kind), value),
            None => format!("Some(\"{}\")", pda_bump_kind_name(&bump.kind)),
        },
        None => "None".to_string(),
    }
}

fn pda_bump_kind_variant_name(kind: &NormalizedPdaBumpKind) -> &'static str {
    match kind {
        NormalizedPdaBumpKind::Canonical => "Canonical",
        NormalizedPdaBumpKind::Arg => "Arg",
    }
}

fn pda_bump_kind_name(kind: &NormalizedPdaBumpKind) -> &'static str {
    match kind {
        NormalizedPdaBumpKind::Canonical => "canonical",
        NormalizedPdaBumpKind::Arg => "arg",
    }
}

fn account_owner_variant_name(owner: &NormalizedAccountOwner) -> &'static str {
    match owner {
        NormalizedAccountOwner::Program => "Program",
        NormalizedAccountOwner::SystemProgram => "SystemProgram",
        NormalizedAccountOwner::TokenProgram => "TokenProgram",
    }
}

fn owner_literal_name(owner: &NormalizedAccountOwner) -> &'static str {
    match owner {
        NormalizedAccountOwner::Program => "program",
        NormalizedAccountOwner::SystemProgram => "system_program",
        NormalizedAccountOwner::TokenProgram => "token_program",
    }
}

fn instruction_account_role_variant_name(
    role: &aqami_spec::InstructionAccountRole,
) -> &'static str {
    match role {
        aqami_spec::InstructionAccountRole::Account => "Account",
        aqami_spec::InstructionAccountRole::Signer => "Signer",
        aqami_spec::InstructionAccountRole::SystemProgram => "SystemProgram",
        aqami_spec::InstructionAccountRole::TokenProgram => "TokenProgram",
        aqami_spec::InstructionAccountRole::Sysvar => "Sysvar",
    }
}

fn option_str_literal(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("Some(\"{value}\")"),
        None => "None".to_string(),
    }
}

fn option_account_owner_literal(account: &NormalizedInstructionAccount) -> String {
    match account.owner.as_ref() {
        Some(owner) => format!("Some(AccountOwner::{})", account_owner_variant_name(owner)),
        None => "None".to_string(),
    }
}

fn option_u64_literal(value: Option<u64>) -> String {
    match value {
        Some(value) => format!("Some({value})"),
        None => "None".to_string(),
    }
}

fn render_constraint_literal(account: &NormalizedInstructionAccount) -> String {
    match account.constraints.as_ref() {
        Some(constraints) => format!(
            "Some(InstructionAccountConstraintDescriptor {{ init: {}, payer: {}, close_to: {}, rent_exempt: {}, has_one: {} }})",
            constraints.init,
            option_str_literal(constraints.payer.as_deref()),
            option_str_literal(constraints.close_to.as_deref()),
            constraints.rent_exempt,
            render_has_one_literal(&constraints.has_one),
        ),
        None => "None".to_string(),
    }
}

fn render_has_one_literal(constraints: &[aqami_spec::NormalizedHasOneConstraint]) -> String {
    if constraints.is_empty() {
        return "&[]".to_string();
    }

    let members = constraints
        .iter()
        .map(|constraint| {
            format!(
                "HasOneConstraintDescriptor {{ field: \"{}\", account: \"{}\" }}",
                constraint.field, constraint.account
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("&[{members}]")
}

fn instruction_account_role_name(role: &aqami_spec::InstructionAccountRole) -> &'static str {
    match role {
        aqami_spec::InstructionAccountRole::Account => "account",
        aqami_spec::InstructionAccountRole::Signer => "signer",
        aqami_spec::InstructionAccountRole::SystemProgram => "system_program",
        aqami_spec::InstructionAccountRole::TokenProgram => "token_program",
        aqami_spec::InstructionAccountRole::Sysvar => "sysvar",
    }
}

fn escape_toml_basic_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::{Path, PathBuf},
    };

    use aqami_spec::{load_project_spec, normalize_project_spec};
    use tempfile::{TempDir, tempdir};

    use super::*;

    const GOLDEN_RUNTIME_PATH: &str = "/aqami/test/aqami-runtime";

    fn escrow_spec_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/specs/escrow.aqami.yaml")
    }

    fn escrow_fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/rust_program/escrow")
    }

    fn generate_escrow_program_fixture() -> (TempDir, Vec<GeneratedProgram>) {
        let loaded = load_project_spec(escrow_spec_path()).expect("example should load");
        let normalized = normalize_project_spec(&loaded.project).expect("example should normalize");
        let temp_dir = tempdir().expect("temp dir should be created");
        let options = GenerateRustProgramOptions {
            aqami_runtime_path: PathBuf::from(GOLDEN_RUNTIME_PATH),
        };
        let generated = generate_rust_programs(&normalized, temp_dir.path(), &options)
            .expect("generation should succeed");

        (temp_dir, generated)
    }

    fn read_relative_text_files(root: &Path) -> BTreeMap<String, String> {
        let mut files = BTreeMap::new();
        read_relative_text_files_into(root, root, &mut files);
        files
    }

    fn read_relative_text_files_into(
        root: &Path,
        dir: &Path,
        files: &mut BTreeMap<String, String>,
    ) {
        let mut entries = fs::read_dir(dir)
            .expect("directory should be readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("directory entries should be readable");
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                read_relative_text_files_into(root, &path, files);
                continue;
            }

            let relative_path = path
                .strip_prefix(root)
                .expect("path should stay under root")
                .to_string_lossy()
                .replace('\\', "/");
            let contents = fs::read_to_string(&path).expect("fixture file should be readable");
            files.insert(relative_path, contents);
        }
    }

    fn generated_relative_paths(program: &GeneratedProgram) -> BTreeSet<String> {
        program
            .files
            .iter()
            .map(|path| {
                path.strip_prefix(&program.output_dir)
                    .expect("generated file should stay under output dir")
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    #[test]
    fn generates_expected_project_files() {
        let (_temp_dir, generated) = generate_escrow_program_fixture();

        assert_eq!(generated.len(), 1);
        let output_dir = &generated[0].output_dir;
        let cargo_toml = fs::read_to_string(output_dir.join("Cargo.toml"))
            .expect("generated Cargo.toml should exist");
        let lib_rs = fs::read_to_string(output_dir.join("src/lib.rs"))
            .expect("generated lib.rs should exist");
        let instruction_rs =
            fs::read_to_string(output_dir.join("src/instructions/create_escrow.rs"))
                .expect("generated instruction should exist");
        let account_rs = fs::read_to_string(output_dir.join("src/state/escrow.rs"))
            .expect("generated account should exist");
        let pda_rs = fs::read_to_string(output_dir.join("src/pdas.rs"))
            .expect("generated pdas.rs should exist");

        assert!(cargo_toml.contains(GOLDEN_RUNTIME_PATH));
        assert!(lib_rs.contains("pub mod instructions;"));
        assert!(!lib_rs.contains("pub mod types;"));
        assert!(instruction_rs.contains("pub struct CreateEscrowAccounts"));
        assert!(instruction_rs.contains("pub const ACCOUNT_DESCRIPTORS"));
        assert!(instruction_rs.contains("pub const PDA_DESCRIPTORS"));
        assert!(instruction_rs.contains("pub fn validate_runtime_accounts("));
        assert!(instruction_rs.contains("validate_program_account_infos_with_pdas"));
        assert!(instruction_rs.contains("pub fn validate_account_descriptors()"));
        assert!(instruction_rs.contains("account_type=Escrow"));
        assert!(instruction_rs.contains("space=128"));
        assert!(account_rs.contains("pub const ACCOUNT_TYPE_DESCRIPTOR: AccountTypeDescriptor"));
        assert!(account_rs.contains("space: Some(128)"));
        assert!(pda_rs.contains("pub const ESCROW_PDA_DESCRIPTOR: PdaDescriptor"));
        assert!(pda_rs.contains("PdaBumpKindDescriptor::Canonical"));
        assert!(instruction_rs.contains("todo!(\"Implement create_escrow\")"));
    }

    #[test]
    fn generated_escrow_program_matches_golden_fixture() {
        let (_temp_dir, generated) = generate_escrow_program_fixture();

        assert_eq!(generated.len(), 1);
        let actual_files = read_relative_text_files(&generated[0].output_dir);
        let expected_files = read_relative_text_files(&escrow_fixture_dir());

        assert_eq!(actual_files, expected_files);
        assert_eq!(
            generated_relative_paths(&generated[0]),
            expected_files.keys().cloned().collect()
        );
    }
}
