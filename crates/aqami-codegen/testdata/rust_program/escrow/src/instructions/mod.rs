pub mod create_escrow;
pub mod release_escrow;

use aqami_runtime::{AccountInfo, Pubkey, RuntimeValidationError, SolanaPubkey};
use std::str;

pub const CREATE_ESCROW_DISCRIMINANT: u8 = 0;
pub const RELEASE_ESCROW_DISCRIMINANT: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowInstructionData {
    CreateEscrow(create_escrow::CreateEscrowArgs),
    ReleaseEscrow(release_escrow::ReleaseEscrowArgs),
}

impl EscrowInstructionData {
    pub fn discriminant(&self) -> u8 {
        match self {
            Self::CreateEscrow(_) => CREATE_ESCROW_DISCRIMINANT,
            Self::ReleaseEscrow(_) => RELEASE_ESCROW_DISCRIMINANT,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Self::CreateEscrow(_) => "create_escrow",
            Self::ReleaseEscrow(_) => "release_escrow",
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut output = vec![self.discriminant()];
        match self {
            Self::CreateEscrow(args) => {
                output.extend_from_slice(&args.depositor);
                output.extend_from_slice(&args.beneficiary);
                output.extend_from_slice(&args.amount.to_le_bytes());
            }
            Self::ReleaseEscrow(_) => {
            }
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowInstructionDataError {
    EmptyInput,
    UnknownDiscriminant { discriminant: u8 },
    UnexpectedEof { needed: usize, remaining: usize },
    UnexpectedTrailingBytes { remaining: usize },
    InvalidBool { value: u8 },
    InvalidUtf8,
}

struct ByteCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ByteCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self { Self { bytes, offset: 0 } }
    fn remaining(&self) -> usize { self.bytes.len().saturating_sub(self.offset) }
    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], EscrowInstructionDataError> {
        if self.remaining() < len {
            return Err(EscrowInstructionDataError::UnexpectedEof { needed: len, remaining: self.remaining() });
        }
        let start = self.offset;
        self.offset += len;
        Ok(&self.bytes[start..self.offset])
    }
}

fn read_u64(cursor: &mut ByteCursor<'_>) -> Result<u64, EscrowInstructionDataError> {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(cursor.read_exact(8)?);
    Ok(u64::from_le_bytes(bytes))
}

fn read_pubkey(cursor: &mut ByteCursor<'_>) -> Result<Pubkey, EscrowInstructionDataError> {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(cursor.read_exact(32)?);
    Ok(bytes)
}

pub fn decode_instruction_data(input: &[u8]) -> Result<EscrowInstructionData, EscrowInstructionDataError> {
    let Some((&discriminant, payload)) = input.split_first() else {
        return Err(EscrowInstructionDataError::EmptyInput);
    };
    let mut cursor = ByteCursor::new(payload);
    let instruction = match discriminant {
        CREATE_ESCROW_DISCRIMINANT => {
            let depositor = read_pubkey(&mut cursor)?;
            let beneficiary = read_pubkey(&mut cursor)?;
            let amount = read_u64(&mut cursor)?;
            EscrowInstructionData::CreateEscrow(create_escrow::CreateEscrowArgs {
                depositor,
                beneficiary,
                amount,
            })
        }
        RELEASE_ESCROW_DISCRIMINANT => {
            EscrowInstructionData::ReleaseEscrow(release_escrow::ReleaseEscrowArgs {
            })
        }
        _ => return Err(EscrowInstructionDataError::UnknownDiscriminant { discriminant }),
    };
    if cursor.remaining() != 0 {
        return Err(EscrowInstructionDataError::UnexpectedTrailingBytes { remaining: cursor.remaining() });
    }
    Ok(instruction)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowInstructionAccountData {
    CreateEscrow,
    ReleaseEscrow {
        account_data: release_escrow::ReleaseEscrowAccountData,
    },
}

impl EscrowInstructionAccountData {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CreateEscrow => "create_escrow",
            Self::ReleaseEscrow { .. } => "release_escrow",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowInstruction {
    CreateEscrow {
        args: create_escrow::CreateEscrowArgs,
    },
    ReleaseEscrow {
        args: release_escrow::ReleaseEscrowArgs,
        account_data: release_escrow::ReleaseEscrowAccountData,
    },
}

impl EscrowInstruction {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CreateEscrow { .. } => "create_escrow",
            Self::ReleaseEscrow { .. } => "release_escrow",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowBindError {
    InstructionAccountDataMismatch { instruction: &'static str, account_data: &'static str },
}

pub fn bind_instruction_context(instruction_data: EscrowInstructionData, account_data: EscrowInstructionAccountData) -> Result<EscrowInstruction, EscrowBindError> {
    let instruction_name = instruction_data.name();
    let account_data_name = account_data.name();
    match (instruction_data, account_data) {
        (EscrowInstructionData::CreateEscrow(args), EscrowInstructionAccountData::CreateEscrow) => Ok(EscrowInstruction::CreateEscrow { args }),
        (EscrowInstructionData::ReleaseEscrow(args), EscrowInstructionAccountData::ReleaseEscrow { account_data }) => Ok(EscrowInstruction::ReleaseEscrow { args, account_data }),
        _ => Err(EscrowBindError::InstructionAccountDataMismatch { instruction: instruction_name, account_data: account_data_name }),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EscrowPreparedInstruction {
    CreateEscrow(create_escrow::CreateEscrowPreparedExecution),
    ReleaseEscrow(release_escrow::ReleaseEscrowPreparedExecution),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EscrowDispatchError {
    CreateEscrow(RuntimeValidationError),
    ReleaseEscrow(RuntimeValidationError),
}

pub fn dispatch_prepare_execution(program_id: &SolanaPubkey, instruction: EscrowInstruction, account_infos: &[AccountInfo<'_>]) -> Result<EscrowPreparedInstruction, EscrowDispatchError> {
    match instruction {
        EscrowInstruction::CreateEscrow { args } => create_escrow::prepare_execution(program_id, account_infos, &args).map(EscrowPreparedInstruction::CreateEscrow).map_err(EscrowDispatchError::CreateEscrow),
        EscrowInstruction::ReleaseEscrow { args, account_data } => release_escrow::prepare_execution(program_id, account_infos, &args, account_data).map(EscrowPreparedInstruction::ReleaseEscrow).map_err(EscrowDispatchError::ReleaseEscrow),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EscrowDispatchFromBytesError {
    Decode(EscrowInstructionDataError),
    Bind(EscrowBindError),
    Dispatch(EscrowDispatchError),
}

pub fn dispatch_prepare_execution_from_bytes(program_id: &SolanaPubkey, instruction_bytes: &[u8], account_data: EscrowInstructionAccountData, account_infos: &[AccountInfo<'_>]) -> Result<EscrowPreparedInstruction, EscrowDispatchFromBytesError> {
    let instruction_data = decode_instruction_data(instruction_bytes).map_err(EscrowDispatchFromBytesError::Decode)?;
    let instruction = bind_instruction_context(instruction_data, account_data).map_err(EscrowDispatchFromBytesError::Bind)?;
    dispatch_prepare_execution(program_id, instruction, account_infos).map_err(EscrowDispatchFromBytesError::Dispatch)
}

#[derive(Debug, PartialEq, Eq)]
pub enum EscrowDispatchFromBytesWithResolverError<ResolveError> {
    Decode(EscrowInstructionDataError),
    Resolve(ResolveError),
    Bind(EscrowBindError),
    Dispatch(EscrowDispatchError),
}

pub fn dispatch_prepare_execution_from_bytes_with_resolver<ResolveError, ResolveAccountData>(program_id: &SolanaPubkey, instruction_bytes: &[u8], account_infos: &[AccountInfo<'_>], resolve_account_data: ResolveAccountData) -> Result<EscrowPreparedInstruction, EscrowDispatchFromBytesWithResolverError<ResolveError>>
where ResolveAccountData: FnOnce(&EscrowInstructionData) -> Result<EscrowInstructionAccountData, ResolveError>,
{
    let instruction_data = decode_instruction_data(instruction_bytes).map_err(EscrowDispatchFromBytesWithResolverError::Decode)?;
    let account_data = resolve_account_data(&instruction_data).map_err(EscrowDispatchFromBytesWithResolverError::Resolve)?;
    let instruction = bind_instruction_context(instruction_data, account_data).map_err(EscrowDispatchFromBytesWithResolverError::Bind)?;
    dispatch_prepare_execution(program_id, instruction, account_infos).map_err(EscrowDispatchFromBytesWithResolverError::Dispatch)
}
