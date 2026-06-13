use std::{
    collections::BTreeSet,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use aqami_spec::{
    NormalizedAccount, NormalizedAccountOwner, NormalizedError, NormalizedEvent,
    NormalizedInstruction, NormalizedInstructionAccount, NormalizedPda, NormalizedProgram,
    NormalizedProjectSpec, SeedKind,
};
use heck::ToUpperCamelCase;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedProgram {
    pub program_name: String,
    pub output_dir: PathBuf,
    pub files: Vec<PathBuf>,
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
) -> Result<Vec<GeneratedProgram>, GenerateError> {
    let output_root = output_root.as_ref();
    let mut generated_programs = Vec::with_capacity(project.programs.len());

    for program in &project.programs {
        generated_programs.push(generate_rust_program(output_root, project, program)?);
    }

    Ok(generated_programs)
}

fn generate_rust_program(
    output_root: &Path,
    project: &NormalizedProjectSpec,
    program: &NormalizedProgram,
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
        &render_cargo_toml(project, program),
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
    write_file(&src_dir.join("types.rs"), &render_types_rs(), &mut files)?;
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
            &render_instruction_rs(instruction),
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

fn render_cargo_toml(project: &NormalizedProjectSpec, program: &NormalizedProgram) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"lib\"]\n",
        program.rust_crate_name, project.package.version
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
    writeln!(&mut output, "pub mod types;").expect("string write should succeed");
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

fn render_types_rs() -> String {
    let mut output = String::new();
    writeln!(&mut output, "pub type Pubkey = [u8; 32];").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(&mut output, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(&mut output, "pub enum AccountOwner {{").expect("string write should succeed");
    writeln!(&mut output, "    Program,").expect("string write should succeed");
    writeln!(&mut output, "    SystemProgram,").expect("string write should succeed");
    writeln!(&mut output, "    TokenProgram,").expect("string write should succeed");
    writeln!(&mut output, "}}").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(&mut output, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(&mut output, "pub enum InstructionAccountRoleDescriptor {{")
        .expect("string write should succeed");
    writeln!(&mut output, "    Account,").expect("string write should succeed");
    writeln!(&mut output, "    Signer,").expect("string write should succeed");
    writeln!(&mut output, "    SystemProgram,").expect("string write should succeed");
    writeln!(&mut output, "    TokenProgram,").expect("string write should succeed");
    writeln!(&mut output, "    Sysvar,").expect("string write should succeed");
    writeln!(&mut output, "}}").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(&mut output, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(
        &mut output,
        "pub struct InstructionAccountConstraintDescriptor {{"
    )
    .expect("string write should succeed");
    writeln!(&mut output, "    pub init: bool,").expect("string write should succeed");
    writeln!(&mut output, "    pub payer: Option<&'static str>,")
        .expect("string write should succeed");
    writeln!(&mut output, "    pub close_to: Option<&'static str>,")
        .expect("string write should succeed");
    writeln!(&mut output, "    pub rent_exempt: bool,").expect("string write should succeed");
    writeln!(&mut output, "}}").expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(&mut output, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        .expect("string write should succeed");
    writeln!(&mut output, "pub struct InstructionAccountDescriptor {{")
        .expect("string write should succeed");
    writeln!(&mut output, "    pub name: &'static str,").expect("string write should succeed");
    writeln!(
        &mut output,
        "    pub role: InstructionAccountRoleDescriptor,"
    )
    .expect("string write should succeed");
    writeln!(&mut output, "    pub account_type: Option<&'static str>,")
        .expect("string write should succeed");
    writeln!(&mut output, "    pub owner: Option<AccountOwner>,")
        .expect("string write should succeed");
    writeln!(&mut output, "    pub is_mut: bool,").expect("string write should succeed");
    writeln!(&mut output, "    pub is_signer: bool,").expect("string write should succeed");
    writeln!(&mut output, "    pub pda: Option<&'static str>,")
        .expect("string write should succeed");
    writeln!(
        &mut output,
        "    pub constraints: Option<InstructionAccountConstraintDescriptor>,"
    )
    .expect("string write should succeed");
    writeln!(&mut output, "}}").expect("string write should succeed");
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

    writeln!(&mut output, "use crate::types::Pubkey;").expect("string write should succeed");
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
    writeln!(&mut output, "use crate::types::{{AccountOwner, Pubkey}};")
        .expect("string write should succeed");
    writeln!(&mut output).expect("string write should succeed");
    writeln!(
        &mut output,
        "pub const OWNER: AccountOwner = AccountOwner::{};",
        account_owner_variant_name(&account.owner)
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

fn render_instruction_rs(instruction: &NormalizedInstruction) -> String {
    let mut output = String::new();
    writeln!(&mut output, "use crate::errors::ProgramError;").expect("string write should succeed");
    let mut type_imports = vec![
        "InstructionAccountDescriptor",
        "InstructionAccountRoleDescriptor",
        "Pubkey",
    ];
    if instruction
        .accounts
        .iter()
        .any(|account| account.owner.is_some())
    {
        type_imports.push("AccountOwner");
    }
    if instruction
        .accounts
        .iter()
        .any(|account| account.constraints.is_some())
    {
        type_imports.push("InstructionAccountConstraintDescriptor");
    }
    writeln!(
        &mut output,
        "use crate::types::{{{}}};",
        type_imports.join(", ")
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

    writeln!(&mut output).expect("string write should succeed");
    writeln!(
        &mut output,
        "pub const ACCOUNT_DESCRIPTORS: &[InstructionAccountDescriptor] = &["
    )
    .expect("string write should succeed");
    for account in &instruction.accounts {
        writeln!(
            &mut output,
            "    InstructionAccountDescriptor {{ name: \"{}\", role: InstructionAccountRoleDescriptor::{}, account_type: {}, owner: {}, is_mut: {}, is_signer: {}, pda: {}, constraints: {} }},",
            account.name,
            instruction_account_role_variant_name(&account.role),
            option_str_literal(account.account_type.as_deref()),
            option_account_owner_literal(account),
            account.is_mut,
            account.is_signer,
            option_str_literal(account.pda.as_deref()),
            render_constraint_literal(account),
        )
        .expect("string write should succeed");
    }
    writeln!(&mut output, "];").expect("string write should succeed");
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

fn render_constraint_literal(account: &NormalizedInstructionAccount) -> String {
    match account.constraints.as_ref() {
        Some(constraints) => format!(
            "Some(InstructionAccountConstraintDescriptor {{ init: {}, payer: {}, close_to: {}, rent_exempt: {} }})",
            constraints.init,
            option_str_literal(constraints.payer.as_deref()),
            option_str_literal(constraints.close_to.as_deref()),
            constraints.rent_exempt
        ),
        None => "None".to_string(),
    }
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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use aqami_spec::{load_project_spec, normalize_project_spec};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn generates_expected_project_files() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/specs/escrow.aqami.yaml");
        let loaded = load_project_spec(path).expect("example should load");
        let normalized = normalize_project_spec(&loaded.project).expect("example should normalize");
        let temp_dir = tempdir().expect("temp dir should be created");

        let generated = generate_rust_programs(&normalized, temp_dir.path())
            .expect("generation should succeed");

        assert_eq!(generated.len(), 1);
        let output_dir = &generated[0].output_dir;
        let lib_rs = fs::read_to_string(output_dir.join("src/lib.rs"))
            .expect("generated lib.rs should exist");
        let instruction_rs =
            fs::read_to_string(output_dir.join("src/instructions/create_escrow.rs"))
                .expect("generated instruction should exist");
        let account_rs = fs::read_to_string(output_dir.join("src/state/escrow.rs"))
            .expect("generated account should exist");

        assert!(lib_rs.contains("pub mod instructions;"));
        assert!(instruction_rs.contains("pub struct CreateEscrowAccounts"));
        assert!(instruction_rs.contains("pub const ACCOUNT_DESCRIPTORS"));
        assert!(instruction_rs.contains("account_type=Escrow"));
        assert!(account_rs.contains("pub const OWNER: AccountOwner = AccountOwner::Program;"));
        assert!(instruction_rs.contains("todo!(\"Implement create_escrow\")"));
    }
}
