use aqami_runtime::{AccountOwner, AccountTypeDescriptor, Pubkey};

pub const ACCOUNT_TYPE_DESCRIPTOR: AccountTypeDescriptor = AccountTypeDescriptor { name: "Escrow", owner: AccountOwner::Program, space: Some(128) };

/// Holds escrow state for a depositor and beneficiary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escrow {
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub status: u8,
}
