use crate::errors::ProgramError;
use aqami_runtime::{AccountInfo, InstructionAccountDescriptor, InstructionAccountRoleDescriptor, Pubkey, RuntimeValidationError, SolanaPubkey, validate_instruction_accounts, AccountOwner, InstructionAccountConstraintDescriptor, HasOneConstraintDescriptor, PdaDescriptor, InstructionAccountPubkeyField, InstructionValidationContext, validate_program_account_infos_with_context};
use crate::state::escrow::Escrow;
use crate::pdas::{ESCROW_PDA_DESCRIPTOR};

pub const ACCOUNT_DESCRIPTORS: &[InstructionAccountDescriptor] = &[
    InstructionAccountDescriptor { name: "depositor", role: InstructionAccountRoleDescriptor::Signer, account_type: None, owner: None, space: None, is_mut: true, is_signer: true, pda: None, constraints: None },
    InstructionAccountDescriptor { name: "beneficiary", role: InstructionAccountRoleDescriptor::Account, account_type: None, owner: None, space: None, is_mut: true, is_signer: false, pda: None, constraints: None },
    InstructionAccountDescriptor { name: "escrow", role: InstructionAccountRoleDescriptor::Account, account_type: Some("Escrow"), owner: Some(AccountOwner::Program), space: Some(128), is_mut: true, is_signer: false, pda: Some("escrow_pda"), constraints: Some(InstructionAccountConstraintDescriptor { init: false, payer: None, close_to: Some("depositor"), rent_exempt: false, has_one: &[HasOneConstraintDescriptor { field: "depositor", account: "depositor" }, HasOneConstraintDescriptor { field: "beneficiary", account: "beneficiary" }] }) },
];

/// PDA descriptors referenced by this instruction's runtime validation path.
pub const PDA_DESCRIPTORS: &[PdaDescriptor] = &[ESCROW_PDA_DESCRIPTOR];

/// Validates real Solana runtime accounts for this instruction against AQAMI descriptors.
pub fn validate_runtime_accounts(program_id: &SolanaPubkey, account_infos: &[AccountInfo<'_>], account_data: &ReleaseEscrowAccountData<'_>) -> Result<(), RuntimeValidationError> {
    let account_pubkey_fields = [
        InstructionAccountPubkeyField { account: "escrow", field: "depositor", value: account_data.escrow.depositor },
        InstructionAccountPubkeyField { account: "escrow", field: "beneficiary", value: account_data.escrow.beneficiary },
    ];
    validate_program_account_infos_with_context(program_id, ACCOUNT_DESCRIPTORS, account_infos, PDA_DESCRIPTORS, &InstructionValidationContext { args: &[], account_pubkey_fields: &account_pubkey_fields })
}

/// Validates descriptor-to-descriptor AQAMI invariants for this instruction.
pub fn validate_account_descriptors() -> Result<(), RuntimeValidationError> {
    validate_instruction_accounts(ACCOUNT_DESCRIPTORS)
}

/// Releases funds to the beneficiary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseEscrowAccounts {
    /// role=signer, mut, signer
    pub depositor: Pubkey,
    /// role=account, mut
    pub beneficiary: Pubkey,
    /// role=account, account_type=Escrow, owner=program, space=128, mut, pda=escrow_pda, close_to=depositor, has_one=depositor->depositor, has_one=beneficiary->beneficiary
    pub escrow: Escrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseEscrowArgs {
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseEscrowAccountData<'a> {
    pub escrow: &'a Escrow,
}

pub fn execute(_accounts: &mut ReleaseEscrowAccounts, _args: ReleaseEscrowArgs) -> Result<(), ProgramError> {
    todo!("Implement release_escrow")
}
