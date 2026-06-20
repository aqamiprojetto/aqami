pub mod create_escrow;
pub mod release_escrow;

use aqami_runtime::{AccountInfo, RuntimeValidationError, SolanaPubkey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowInstruction<'a> {
    CreateEscrow {
        args: &'a create_escrow::CreateEscrowArgs,
    },
    ReleaseEscrow {
        args: &'a release_escrow::ReleaseEscrowArgs,
        account_data: &'a release_escrow::ReleaseEscrowAccountData<'a>,
    },
}

impl EscrowInstruction<'_> {
    pub fn name(&self) -> &'static str {
        match self {
            Self::CreateEscrow { .. } => "create_escrow",
            Self::ReleaseEscrow { .. } => "release_escrow",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EscrowPreparedInstruction<'a> {
    CreateEscrow(create_escrow::CreateEscrowPreparedExecution),
    ReleaseEscrow(release_escrow::ReleaseEscrowPreparedExecution<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EscrowDispatchError {
    CreateEscrow(RuntimeValidationError),
    ReleaseEscrow(RuntimeValidationError),
}

pub fn dispatch_prepare_execution<'a>(program_id: &SolanaPubkey, instruction: EscrowInstruction<'a>, account_infos: &[AccountInfo<'_>]) -> Result<EscrowPreparedInstruction<'a>, EscrowDispatchError> {
    match instruction {
        EscrowInstruction::CreateEscrow { args } => create_escrow::prepare_execution(program_id, account_infos, args).map(EscrowPreparedInstruction::CreateEscrow).map_err(EscrowDispatchError::CreateEscrow),
        EscrowInstruction::ReleaseEscrow { args, account_data } => release_escrow::prepare_execution(program_id, account_infos, args, account_data).map(EscrowPreparedInstruction::ReleaseEscrow).map_err(EscrowDispatchError::ReleaseEscrow),
    }
}
