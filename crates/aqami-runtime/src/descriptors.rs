#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountOwner {
    Program,
    SystemProgram,
    TokenProgram,
}

pub type Pubkey = [u8; 32];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionAccountRoleDescriptor {
    Account,
    Signer,
    SystemProgram,
    TokenProgram,
    Sysvar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountTypeDescriptor {
    pub name: &'static str,
    pub owner: AccountOwner,
    pub space: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionAccountConstraintDescriptor {
    pub init: bool,
    pub payer: Option<&'static str>,
    pub close_to: Option<&'static str>,
    pub rent_exempt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionAccountDescriptor {
    pub name: &'static str,
    pub role: InstructionAccountRoleDescriptor,
    pub account_type: Option<&'static str>,
    pub owner: Option<AccountOwner>,
    pub space: Option<u64>,
    pub is_mut: bool,
    pub is_signer: bool,
    pub pda: Option<&'static str>,
    pub constraints: Option<InstructionAccountConstraintDescriptor>,
}
