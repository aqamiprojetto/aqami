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
pub enum PdaSeedKindDescriptor {
    Const,
    Arg,
    AccountField,
    AccountKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionArgValue<'a> {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    String(&'a str),
    Bytes(&'a [u8]),
    Pubkey(Pubkey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionArg<'a> {
    pub name: &'a str,
    pub value: InstructionArgValue<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdaSeedDescriptor {
    pub kind: PdaSeedKindDescriptor,
    pub value: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdaBumpKindDescriptor {
    Canonical,
    Arg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdaBumpDescriptor {
    pub kind: PdaBumpKindDescriptor,
    pub value: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdaDescriptor {
    pub name: &'static str,
    pub seeds: &'static [PdaSeedDescriptor],
    pub bump: Option<PdaBumpDescriptor>,
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
    pub has_one: &'static [HasOneConstraintDescriptor],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HasOneConstraintDescriptor {
    pub field: &'static str,
    pub account: &'static str,
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
