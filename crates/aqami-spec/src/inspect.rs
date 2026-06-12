use serde::Serialize;

use crate::{
    AccountSpec, AqamiProjectSpec, EventSpec, FrameworkErrorSpec, InstructionSpec, PdaSpec,
    ProgramSpec,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProjectInspection {
    pub spec_version: String,
    pub package_name: String,
    pub package_version: String,
    pub cluster: Option<String>,
    pub program_count: usize,
    pub programs: Vec<ProgramInspection>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProgramInspection {
    pub name: String,
    pub account_count: usize,
    pub instruction_count: usize,
    pub pda_count: usize,
    pub event_count: usize,
    pub error_count: usize,
    pub accounts: Vec<NamedInspection>,
    pub instructions: Vec<InstructionInspection>,
    pub pdas: Vec<NamedInspection>,
    pub events: Vec<NamedInspection>,
    pub errors: Vec<ErrorInspection>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NamedInspection {
    pub name: String,
    pub docs: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InstructionInspection {
    pub name: String,
    pub docs: Option<String>,
    pub account_names: Vec<String>,
    pub arg_names: Vec<String>,
    pub emits: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ErrorInspection {
    pub name: String,
    pub code: i64,
    pub message: String,
}

impl From<&AqamiProjectSpec> for ProjectInspection {
    fn from(project: &AqamiProjectSpec) -> Self {
        Self {
            spec_version: project.spec_version.clone(),
            package_name: project.package.name.clone(),
            package_version: project.package.version.clone(),
            cluster: project.cluster.as_ref().map(cluster_name),
            program_count: project.programs.len(),
            programs: project
                .programs
                .iter()
                .map(ProgramInspection::from)
                .collect(),
        }
    }
}

impl From<&ProgramSpec> for ProgramInspection {
    fn from(program: &ProgramSpec) -> Self {
        Self {
            name: program.name.clone(),
            account_count: program.accounts.len(),
            instruction_count: program.instructions.len(),
            pda_count: program.pdas.len(),
            event_count: program.events.len(),
            error_count: program.errors.len(),
            accounts: program.accounts.iter().map(named_from_account).collect(),
            instructions: program
                .instructions
                .iter()
                .map(InstructionInspection::from)
                .collect(),
            pdas: program.pdas.iter().map(named_from_pda).collect(),
            events: program.events.iter().map(named_from_event).collect(),
            errors: program.errors.iter().map(ErrorInspection::from).collect(),
        }
    }
}

impl From<&InstructionSpec> for InstructionInspection {
    fn from(instruction: &InstructionSpec) -> Self {
        Self {
            name: instruction.name.clone(),
            docs: instruction.docs.clone(),
            account_names: instruction
                .accounts
                .iter()
                .map(|account| account.name.clone())
                .collect(),
            arg_names: instruction
                .args
                .iter()
                .map(|arg| arg.name.clone())
                .collect(),
            emits: instruction.emits.clone(),
            errors: instruction.errors.clone(),
        }
    }
}

impl From<&FrameworkErrorSpec> for ErrorInspection {
    fn from(error: &FrameworkErrorSpec) -> Self {
        Self {
            name: error.name.clone(),
            code: error.code,
            message: error.message.clone(),
        }
    }
}

fn cluster_name(cluster: &crate::Cluster) -> String {
    match cluster {
        crate::Cluster::Localnet => "localnet".to_string(),
        crate::Cluster::Devnet => "devnet".to_string(),
        crate::Cluster::Testnet => "testnet".to_string(),
        crate::Cluster::MainnetBeta => "mainnet-beta".to_string(),
    }
}

fn named_from_account(account: &AccountSpec) -> NamedInspection {
    NamedInspection {
        name: account.name.clone(),
        docs: account.docs.clone(),
    }
}

fn named_from_pda(pda: &PdaSpec) -> NamedInspection {
    NamedInspection {
        name: pda.name.clone(),
        docs: pda.docs.clone(),
    }
}

fn named_from_event(event: &EventSpec) -> NamedInspection {
    NamedInspection {
        name: event.name.clone(),
        docs: None,
    }
}
