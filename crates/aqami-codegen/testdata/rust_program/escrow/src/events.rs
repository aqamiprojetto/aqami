use aqami_runtime::Pubkey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowCreated {
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowReleased {
    pub escrow: Pubkey,
}

