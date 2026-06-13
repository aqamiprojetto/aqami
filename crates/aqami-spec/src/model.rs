use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AqamiProjectSpec {
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    pub package: PackageSpec,
    pub cluster: Option<Cluster>,
    pub programs: Vec<ProgramSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Cluster {
    Localnet,
    Devnet,
    Testnet,
    MainnetBeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageSpec {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProgramSpec {
    pub name: String,
    #[serde(rename = "programId")]
    pub program_id: Option<String>,
    pub docs: Option<String>,
    pub accounts: Vec<AccountSpec>,
    pub instructions: Vec<InstructionSpec>,
    #[serde(default)]
    pub pdas: Vec<PdaSpec>,
    #[serde(default)]
    pub events: Vec<EventSpec>,
    #[serde(default)]
    pub errors: Vec<FrameworkErrorSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSpec {
    pub name: String,
    pub docs: Option<String>,
    pub owner: Option<AccountOwner>,
    pub space: Option<u64>,
    pub fields: Vec<FieldSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub docs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionSpec {
    pub name: String,
    pub docs: Option<String>,
    pub accounts: Vec<InstructionAccountSpec>,
    #[serde(default)]
    pub args: Vec<FieldSpec>,
    #[serde(default)]
    pub emits: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstructionAccountSpec {
    pub name: String,
    pub role: InstructionAccountRole,
    #[serde(rename = "accountType")]
    pub account_type: Option<String>,
    pub constraints: Option<InstructionAccountConstraintsSpec>,
    #[serde(default)]
    pub is_mut: bool,
    #[serde(default)]
    pub is_signer: bool,
    pub pda: Option<String>,
    pub docs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstructionAccountRole {
    Account,
    Signer,
    SystemProgram,
    TokenProgram,
    Sysvar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountOwner {
    Program,
    SystemProgram,
    TokenProgram,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstructionAccountConstraintsSpec {
    #[serde(default)]
    pub init: bool,
    pub payer: Option<String>,
    pub close_to: Option<String>,
    #[serde(default)]
    pub rent_exempt: bool,
    #[serde(default)]
    pub has_one: Vec<HasOneConstraintSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HasOneConstraintSpec {
    pub field: String,
    pub account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PdaSpec {
    pub name: String,
    pub docs: Option<String>,
    pub seeds: Vec<SeedSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedSpec {
    pub kind: SeedKind,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeedKind {
    Const,
    Arg,
    AccountField,
    AccountKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventSpec {
    pub name: String,
    pub fields: Vec<FieldSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameworkErrorSpec {
    pub name: String,
    pub code: i64,
    pub message: String,
}
