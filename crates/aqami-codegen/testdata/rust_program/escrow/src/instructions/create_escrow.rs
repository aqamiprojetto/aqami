use crate::errors::ProgramError;
use aqami_runtime::{AccountInfo, InstructionAccountDescriptor, InstructionAccountRoleDescriptor, Pubkey, RuntimeValidationError, SolanaPubkey, validate_instruction_accounts, AccountOwner, InstructionAccountConstraintDescriptor, PdaDescriptor, validate_program_account_infos_with_pdas};
use crate::state::escrow::Escrow;
use crate::pdas::{ESCROW_PDA_DESCRIPTOR};

pub const ACCOUNT_DESCRIPTORS: &[InstructionAccountDescriptor] = &[
    InstructionAccountDescriptor { name: "depositor", role: InstructionAccountRoleDescriptor::Signer, account_type: None, owner: None, space: None, is_mut: false, is_signer: true, pda: None, constraints: None },
    InstructionAccountDescriptor { name: "beneficiary", role: InstructionAccountRoleDescriptor::Account, account_type: None, owner: None, space: None, is_mut: false, is_signer: false, pda: None, constraints: None },
    InstructionAccountDescriptor { name: "escrow", role: InstructionAccountRoleDescriptor::Account, account_type: Some("Escrow"), owner: Some(AccountOwner::Program), space: Some(128), is_mut: true, is_signer: false, pda: Some("escrow_pda"), constraints: Some(InstructionAccountConstraintDescriptor { init: true, payer: Some("depositor"), close_to: None, rent_exempt: true, has_one: &[] }) },
    InstructionAccountDescriptor { name: "system_program", role: InstructionAccountRoleDescriptor::SystemProgram, account_type: None, owner: None, space: None, is_mut: false, is_signer: false, pda: None, constraints: None },
];

/// PDA descriptors referenced by this instruction's runtime validation path.
pub const PDA_DESCRIPTORS: &[PdaDescriptor] = &[ESCROW_PDA_DESCRIPTOR];

/// Validates real Solana runtime accounts for this instruction against AQAMI descriptors.
pub fn validate_runtime_accounts(program_id: &SolanaPubkey, account_infos: &[AccountInfo<'_>]) -> Result<(), RuntimeValidationError> {
    validate_program_account_infos_with_pdas(program_id, ACCOUNT_DESCRIPTORS, account_infos, PDA_DESCRIPTORS)
}

/// Validates descriptor-to-descriptor AQAMI invariants for this instruction.
pub fn validate_account_descriptors() -> Result<(), RuntimeValidationError> {
    validate_instruction_accounts(ACCOUNT_DESCRIPTORS)
}

/// Creates a new escrow agreement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateEscrowAccounts {
    /// role=signer, signer
    pub depositor: Pubkey,
    /// role=account
    pub beneficiary: Pubkey,
    /// role=account, account_type=Escrow, owner=program, space=128, mut, pda=escrow_pda, init, payer=depositor, rent_exempt
    pub escrow: Escrow,
    /// role=system_program
    pub system_program: Pubkey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateEscrowArgs {
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub amount: u64,
}

pub fn execute(_accounts: &mut CreateEscrowAccounts, _args: CreateEscrowArgs) -> Result<(), ProgramError> {
    todo!("Implement create_escrow")
}
