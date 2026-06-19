mod descriptors;
mod validate;

pub use solana_program::{account_info::AccountInfo, pubkey::Pubkey as SolanaPubkey};

pub use descriptors::{
    AccountOwner, AccountTypeDescriptor, HasOneConstraintDescriptor,
    InstructionAccountConstraintDescriptor, InstructionAccountDescriptor,
    InstructionAccountRoleDescriptor, PdaBumpDescriptor, PdaBumpKindDescriptor, PdaDescriptor,
    PdaSeedDescriptor, PdaSeedKindDescriptor, Pubkey,
};
pub use validate::{
    RuntimeValidationError, validate_account_infos, validate_instruction_accounts,
    validate_program_account_infos, validate_program_account_infos_with_pdas,
};
