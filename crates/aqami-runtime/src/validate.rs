use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey as SolanaPubkey,
};
use solana_system_interface::program as system_program;
use thiserror::Error;

use crate::{
    AccountOwner, InstructionAccountDescriptor, InstructionAccountPubkeyField,
    InstructionAccountRoleDescriptor, InstructionArg, InstructionArgValue,
    InstructionValidationContext, PdaBumpKindDescriptor, PdaDescriptor, PdaSeedKindDescriptor,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuntimeValidationError {
    #[error("instruction account count mismatch: expected {expected}, got {actual}")]
    AccountCountMismatch { expected: usize, actual: usize },
    #[error("runtime account `{account}` must be a signer")]
    MissingRequiredSignature { account: &'static str },
    #[error("runtime account `{account}` must be writable")]
    AccountNotMutable { account: &'static str },
    #[error(
        "runtime account `{account}` has owner `{actual_owner}` but expected `{expected_owner}`"
    )]
    AccountOwnerMismatch {
        account: &'static str,
        expected_owner: SolanaPubkey,
        actual_owner: SolanaPubkey,
    },
    #[error("runtime account `{account}` must be the system program, got `{actual_key}`")]
    IncorrectSystemProgramAccount {
        account: &'static str,
        actual_key: SolanaPubkey,
    },
    #[error(
        "runtime account `{account}` uses token-program ownership semantics that AQAMI does not yet validate explicitly"
    )]
    UnsupportedTokenProgramOwnerValidation { account: &'static str },
    #[error("instruction account `{account}` references unknown PDA descriptor `{pda}`")]
    UnknownPdaDescriptor {
        account: &'static str,
        pda: &'static str,
    },
    #[error("PDA `{pda}` for account `{account}` references unknown seed account `{seed_account}`")]
    UnknownPdaSeedAccount {
        account: &'static str,
        pda: &'static str,
        seed_account: &'static str,
    },
    #[error("PDA `{pda}` for account `{account}` references unknown instruction argument `{arg}`")]
    UnknownPdaSeedArg {
        account: &'static str,
        pda: &'static str,
        arg: &'static str,
    },
    #[error(
        "runtime validation requires pubkey field `{account}.{field}` but no value was supplied"
    )]
    MissingAccountPubkeyField {
        account: &'static str,
        field: &'static str,
    },
    #[error("PDA `{pda}` for account `{account}` uses unsupported seed kind `{kind}`")]
    UnsupportedPdaSeedKind {
        account: &'static str,
        pda: &'static str,
        kind: &'static str,
    },
    #[error("PDA `{pda}` for account `{account}` references unknown bump argument `{arg}`")]
    UnknownPdaBumpArg {
        account: &'static str,
        pda: &'static str,
        arg: &'static str,
    },
    #[error("PDA `{pda}` for account `{account}` uses unsupported bump kind `{kind}`")]
    UnsupportedPdaBumpKind {
        account: &'static str,
        pda: &'static str,
        kind: &'static str,
    },
    #[error(
        "PDA `{pda}` for account `{account}` uses bump argument `{arg}` with unsupported AQAMI type `{kind}`"
    )]
    InvalidPdaBumpArgType {
        account: &'static str,
        pda: &'static str,
        arg: &'static str,
        kind: &'static str,
    },
    #[error("PDA `{pda}` for account `{account}` could not be derived for this program")]
    PdaDerivationFailed {
        account: &'static str,
        pda: &'static str,
    },
    #[error(
        "runtime PDA account `{account}` has key `{actual_key}` but expected `{expected_key}` from `{pda}`"
    )]
    PdaMismatch {
        account: &'static str,
        pda: &'static str,
        expected_key: SolanaPubkey,
        actual_key: SolanaPubkey,
    },
    #[error(
        "runtime `has_one` check failed for `{account}.{field}`: expected key `{expected_key}` from instruction account `{related_account}`, got `{actual_key}`"
    )]
    HasOneMismatch {
        account: &'static str,
        field: &'static str,
        related_account: &'static str,
        expected_key: SolanaPubkey,
        actual_key: SolanaPubkey,
    },
    #[error("initialized account `{account}` must be mutable")]
    InitWithoutMutability { account: &'static str },
    #[error("initialized account `{account}` must declare a payer")]
    InitWithoutPayer { account: &'static str },
    #[error("payer `{payer}` for account `{account}` does not exist")]
    UnknownPayer {
        account: &'static str,
        payer: &'static str,
    },
    #[error("payer `{payer}` for account `{account}` must be a signer")]
    PayerMustBeSigner {
        account: &'static str,
        payer: &'static str,
    },
    #[error("program-owned initialized account `{account}` must declare space")]
    InitWithoutSpace { account: &'static str },
    #[error("closed account `{account}` must be mutable")]
    CloseWithoutMutability { account: &'static str },
    #[error("close target `{target}` for account `{account}` does not exist")]
    UnknownCloseTarget {
        account: &'static str,
        target: &'static str,
    },
    #[error("close target `{target}` for account `{account}` must be mutable")]
    CloseTargetMustBeMutable {
        account: &'static str,
        target: &'static str,
    },
    #[error("account `{account}` cannot declare both init and close semantics")]
    InitAndCloseConflict { account: &'static str },
    #[error(
        "`has_one` on account `{account}` references unknown instruction account `{related_account}`"
    )]
    UnknownHasOneAccount {
        account: &'static str,
        related_account: &'static str,
    },
}

/// Validates descriptor-to-descriptor invariants before any real Solana account data is involved.
pub fn validate_instruction_accounts(
    accounts: &[InstructionAccountDescriptor],
) -> Result<(), RuntimeValidationError> {
    for account in accounts {
        let Some(constraints) = account.constraints else {
            continue;
        };

        if constraints.init {
            if !account.is_mut {
                return Err(RuntimeValidationError::InitWithoutMutability {
                    account: account.name,
                });
            }

            let Some(payer_name) = constraints.payer else {
                return Err(RuntimeValidationError::InitWithoutPayer {
                    account: account.name,
                });
            };

            let Some(payer_account) = accounts
                .iter()
                .find(|candidate| candidate.name == payer_name)
            else {
                return Err(RuntimeValidationError::UnknownPayer {
                    account: account.name,
                    payer: payer_name,
                });
            };

            if !payer_account.is_signer {
                return Err(RuntimeValidationError::PayerMustBeSigner {
                    account: account.name,
                    payer: payer_name,
                });
            }

            if matches!(account.owner, Some(AccountOwner::Program)) && account.space.is_none() {
                return Err(RuntimeValidationError::InitWithoutSpace {
                    account: account.name,
                });
            }
        }

        if let Some(close_target_name) = constraints.close_to {
            if !account.is_mut {
                return Err(RuntimeValidationError::CloseWithoutMutability {
                    account: account.name,
                });
            }

            if constraints.init {
                return Err(RuntimeValidationError::InitAndCloseConflict {
                    account: account.name,
                });
            }

            let Some(close_target) = accounts
                .iter()
                .find(|candidate| candidate.name == close_target_name)
            else {
                return Err(RuntimeValidationError::UnknownCloseTarget {
                    account: account.name,
                    target: close_target_name,
                });
            };

            if !close_target.is_mut {
                return Err(RuntimeValidationError::CloseTargetMustBeMutable {
                    account: account.name,
                    target: close_target_name,
                });
            }
        }

        for relation in constraints.has_one {
            if !accounts
                .iter()
                .any(|candidate| candidate.name == relation.account)
            {
                return Err(RuntimeValidationError::UnknownHasOneAccount {
                    account: account.name,
                    related_account: relation.account,
                });
            }
        }
    }

    Ok(())
}

/// Validates actual Solana account metadata against AQAMI instruction descriptors.
///
/// This first checks static descriptor invariants, then verifies runtime-facing
/// signer, writable, and account-count expectations from `AccountInfo`.
///
/// This helper intentionally stays metadata-only.
/// Checks that need the active `program_id`, such as program-owned account
/// ownership enforcement, belong in `validate_program_account_infos`.
pub fn validate_account_infos(
    expected: &[InstructionAccountDescriptor],
    actual: &[AccountInfo<'_>],
) -> Result<(), RuntimeValidationError> {
    validate_instruction_accounts(expected)?;

    if expected.len() != actual.len() {
        return Err(RuntimeValidationError::AccountCountMismatch {
            expected: expected.len(),
            actual: actual.len(),
        });
    }

    for (descriptor, account_info) in expected.iter().zip(actual.iter()) {
        if descriptor.is_signer && !account_info.is_signer {
            return Err(RuntimeValidationError::MissingRequiredSignature {
                account: descriptor.name,
            });
        }

        if descriptor.is_mut && !account_info.is_writable {
            return Err(RuntimeValidationError::AccountNotMutable {
                account: descriptor.name,
            });
        }
    }

    Ok(())
}

/// Validates actual Solana account metadata plus program-context ownership rules.
///
/// This extends `validate_account_infos` with checks that require the executing
/// `program_id`, including:
///
/// - program-owned AQAMI account enforcement
/// - system-owned AQAMI account enforcement
/// - exact `system_program` role identity
///
/// AQAMI intentionally returns an explicit error for `token_program` ownership
/// validation today because the current spec model does not yet distinguish
/// classic SPL Token from Token-2022 ownership semantics.
pub fn validate_program_account_infos(
    program_id: &SolanaPubkey,
    expected: &[InstructionAccountDescriptor],
    actual: &[AccountInfo<'_>],
) -> Result<(), RuntimeValidationError> {
    validate_account_infos(expected, actual)?;

    for (descriptor, account_info) in expected.iter().zip(actual.iter()) {
        if matches!(
            descriptor.role,
            InstructionAccountRoleDescriptor::SystemProgram
        ) && account_info.key != &system_program::ID
        {
            return Err(RuntimeValidationError::IncorrectSystemProgramAccount {
                account: descriptor.name,
                actual_key: *account_info.key,
            });
        }

        let expected_owner = match descriptor.owner {
            Some(AccountOwner::Program) => Some(*program_id),
            Some(AccountOwner::SystemProgram) => Some(system_program::ID),
            Some(AccountOwner::TokenProgram) => {
                return Err(
                    RuntimeValidationError::UnsupportedTokenProgramOwnerValidation {
                        account: descriptor.name,
                    },
                );
            }
            None => None,
        };

        if let Some(expected_owner) = expected_owner
            && account_info.owner != &expected_owner
        {
            return Err(RuntimeValidationError::AccountOwnerMismatch {
                account: descriptor.name,
                expected_owner,
                actual_owner: *account_info.owner,
            });
        }
    }

    Ok(())
}

/// Validates program-context account metadata plus PDA semantics that only need
/// account metadata and static PDA descriptors.
pub fn validate_program_account_infos_with_pdas(
    program_id: &SolanaPubkey,
    expected: &[InstructionAccountDescriptor],
    actual: &[AccountInfo<'_>],
    pda_descriptors: &[PdaDescriptor],
) -> Result<(), RuntimeValidationError> {
    validate_program_account_infos_with_context(
        program_id,
        expected,
        actual,
        pda_descriptors,
        &InstructionValidationContext::default(),
    )
}

/// Validates program-context account metadata plus PDA and relationship semantics
/// that depend on explicit AQAMI runtime context.
///
/// This extends metadata-only validation with:
///
/// - `arg` seeds
/// - `arg` bumps
/// - `account_field` PDA seeds backed by supplied pubkey field values
/// - `has_one` checks backed by supplied pubkey field values
///
/// AQAMI intentionally keeps this context explicit rather than decoding raw
/// account bytes implicitly, because AQAMI has not yet defined a stable
/// serialization contract for generic runtime inspection.
pub fn validate_program_account_infos_with_context(
    program_id: &SolanaPubkey,
    expected: &[InstructionAccountDescriptor],
    actual: &[AccountInfo<'_>],
    pda_descriptors: &[PdaDescriptor],
    context: &InstructionValidationContext<'_>,
) -> Result<(), RuntimeValidationError> {
    validate_program_account_infos(program_id, expected, actual)?;

    for descriptor in expected {
        let Some(constraints) = descriptor.constraints else {
            continue;
        };

        for relation in constraints.has_one {
            let Some(actual_field_value) = instruction_account_pubkey_field(
                context.account_pubkey_fields,
                descriptor.name,
                relation.field,
            ) else {
                return Err(RuntimeValidationError::MissingAccountPubkeyField {
                    account: descriptor.name,
                    field: relation.field,
                });
            };
            let Some(related_account_index) = instruction_account_index(expected, relation.account)
            else {
                return Err(RuntimeValidationError::UnknownHasOneAccount {
                    account: descriptor.name,
                    related_account: relation.account,
                });
            };
            let expected_key = *actual[related_account_index].key;
            if *actual_field_value != expected_key.to_bytes() {
                return Err(RuntimeValidationError::HasOneMismatch {
                    account: descriptor.name,
                    field: relation.field,
                    related_account: relation.account,
                    expected_key,
                    actual_key: SolanaPubkey::new_from_array(*actual_field_value),
                });
            }
        }
    }

    for (descriptor, account_info) in expected.iter().zip(actual.iter()) {
        let Some(pda_name) = descriptor.pda else {
            continue;
        };

        let Some(pda_descriptor) = pda_descriptors
            .iter()
            .find(|candidate| candidate.name == pda_name)
        else {
            return Err(RuntimeValidationError::UnknownPdaDescriptor {
                account: descriptor.name,
                pda: pda_name,
            });
        };

        let mut resolved_seeds = Vec::with_capacity(pda_descriptor.seeds.len());
        for seed in pda_descriptor.seeds {
            match seed.kind {
                PdaSeedKindDescriptor::Const => {
                    resolved_seeds.push(ResolvedSeed::Borrowed(seed.value.as_bytes()))
                }
                PdaSeedKindDescriptor::AccountKey => {
                    let Some(seed_account_index) = instruction_account_index(expected, seed.value)
                    else {
                        return Err(RuntimeValidationError::UnknownPdaSeedAccount {
                            account: descriptor.name,
                            pda: pda_name,
                            seed_account: seed.value,
                        });
                    };
                    resolved_seeds.push(ResolvedSeed::Borrowed(
                        actual[seed_account_index].key.as_ref(),
                    ));
                }
                PdaSeedKindDescriptor::Arg => {
                    let Some(seed_arg) = instruction_arg_value(context.args, seed.value) else {
                        return Err(RuntimeValidationError::UnknownPdaSeedArg {
                            account: descriptor.name,
                            pda: pda_name,
                            arg: seed.value,
                        });
                    };
                    resolved_seeds.push(ResolvedSeed::Owned(instruction_arg_seed_bytes(seed_arg)));
                }
                PdaSeedKindDescriptor::AccountField => {
                    let Some((seed_account, seed_field)) = seed.value.split_once('.') else {
                        return Err(RuntimeValidationError::UnsupportedPdaSeedKind {
                            account: descriptor.name,
                            pda: pda_name,
                            kind: pda_seed_kind_name(seed.kind),
                        });
                    };
                    let Some(seed_field_value) = instruction_account_pubkey_field(
                        context.account_pubkey_fields,
                        seed_account,
                        seed_field,
                    ) else {
                        return Err(RuntimeValidationError::MissingAccountPubkeyField {
                            account: seed_account,
                            field: seed_field,
                        });
                    };
                    resolved_seeds.push(ResolvedSeed::Borrowed(seed_field_value.as_ref()));
                }
            }
        }
        let seed_slices = resolved_seeds
            .iter()
            .map(|seed| match seed {
                ResolvedSeed::Borrowed(value) => *value,
                ResolvedSeed::Owned(value) => value.as_slice(),
            })
            .collect::<Vec<_>>();

        let expected_key =
            match pda_descriptor.bump {
                None => SolanaPubkey::create_program_address(&seed_slices, program_id).map_err(
                    |_| RuntimeValidationError::PdaDerivationFailed {
                        account: descriptor.name,
                        pda: pda_name,
                    },
                )?,
                Some(bump) => match bump.kind {
                    PdaBumpKindDescriptor::Canonical => {
                        SolanaPubkey::find_program_address(&seed_slices, program_id).0
                    }
                    PdaBumpKindDescriptor::Arg => {
                        let arg_name = bump.value.expect("arg-backed bumps must declare a value");
                        let Some(bump_arg) = instruction_arg_value(context.args, arg_name) else {
                            return Err(RuntimeValidationError::UnknownPdaBumpArg {
                                account: descriptor.name,
                                pda: pda_name,
                                arg: arg_name,
                            });
                        };
                        let Some(bump_value) = instruction_arg_bump_value(bump_arg) else {
                            return Err(RuntimeValidationError::InvalidPdaBumpArgType {
                                account: descriptor.name,
                                pda: pda_name,
                                arg: arg_name,
                                kind: instruction_arg_value_kind_name(bump_arg),
                            });
                        };
                        let bump_seed = [bump_value];
                        let mut seed_slices_with_bump = seed_slices.clone();
                        seed_slices_with_bump.push(&bump_seed);
                        SolanaPubkey::create_program_address(&seed_slices_with_bump, program_id)
                            .map_err(|_| RuntimeValidationError::PdaDerivationFailed {
                                account: descriptor.name,
                                pda: pda_name,
                            })?
                    }
                },
            };

        if account_info.key != &expected_key {
            return Err(RuntimeValidationError::PdaMismatch {
                account: descriptor.name,
                pda: pda_name,
                expected_key,
                actual_key: *account_info.key,
            });
        }
    }

    Ok(())
}

/// Validates program-context account metadata plus PDA semantics that depend on
/// explicit AQAMI instruction arguments.
pub fn validate_program_account_infos_with_pdas_and_args(
    program_id: &SolanaPubkey,
    expected: &[InstructionAccountDescriptor],
    actual: &[AccountInfo<'_>],
    pda_descriptors: &[PdaDescriptor],
    instruction_args: &[InstructionArg<'_>],
) -> Result<(), RuntimeValidationError> {
    validate_program_account_infos_with_context(
        program_id,
        expected,
        actual,
        pda_descriptors,
        &InstructionValidationContext {
            args: instruction_args,
            account_pubkey_fields: &[],
        },
    )
}

enum ResolvedSeed<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

fn instruction_account_pubkey_field<'a>(
    account_pubkey_fields: &'a [InstructionAccountPubkeyField<'a>],
    account_name: &str,
    field_name: &str,
) -> Option<&'a crate::Pubkey> {
    account_pubkey_fields
        .iter()
        .find(|field| field.account == account_name && field.field == field_name)
        .map(|field| &field.value)
}

fn instruction_arg_value<'a>(
    instruction_args: &'a [InstructionArg<'a>],
    arg_name: &str,
) -> Option<&'a InstructionArgValue<'a>> {
    instruction_args
        .iter()
        .find(|arg| arg.name == arg_name)
        .map(|arg| &arg.value)
}

fn instruction_account_index(
    expected: &[InstructionAccountDescriptor],
    account_name: &str,
) -> Option<usize> {
    expected
        .iter()
        .position(|candidate| candidate.name == account_name)
}

fn instruction_arg_seed_bytes(value: &InstructionArgValue<'_>) -> Vec<u8> {
    match value {
        InstructionArgValue::Bool(value) => vec![u8::from(*value)],
        InstructionArgValue::U8(value) => vec![*value],
        InstructionArgValue::U16(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::U32(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::U64(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::U128(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::I8(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::I16(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::I32(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::I64(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::I128(value) => value.to_le_bytes().to_vec(),
        InstructionArgValue::String(value) => value.as_bytes().to_vec(),
        InstructionArgValue::Bytes(value) => value.to_vec(),
        InstructionArgValue::Pubkey(value) => value.to_vec(),
    }
}

fn instruction_arg_bump_value(value: &InstructionArgValue<'_>) -> Option<u8> {
    match value {
        InstructionArgValue::U8(value) => Some(*value),
        _ => None,
    }
}

fn instruction_arg_value_kind_name(value: &InstructionArgValue<'_>) -> &'static str {
    match value {
        InstructionArgValue::Bool(_) => "bool",
        InstructionArgValue::U8(_) => "u8",
        InstructionArgValue::U16(_) => "u16",
        InstructionArgValue::U32(_) => "u32",
        InstructionArgValue::U64(_) => "u64",
        InstructionArgValue::U128(_) => "u128",
        InstructionArgValue::I8(_) => "i8",
        InstructionArgValue::I16(_) => "i16",
        InstructionArgValue::I32(_) => "i32",
        InstructionArgValue::I64(_) => "i64",
        InstructionArgValue::I128(_) => "i128",
        InstructionArgValue::String(_) => "string",
        InstructionArgValue::Bytes(_) => "bytes",
        InstructionArgValue::Pubkey(_) => "pubkey",
    }
}

fn pda_seed_kind_name(kind: PdaSeedKindDescriptor) -> &'static str {
    match kind {
        PdaSeedKindDescriptor::Const => "const",
        PdaSeedKindDescriptor::Arg => "arg",
        PdaSeedKindDescriptor::AccountField => "account_field",
        PdaSeedKindDescriptor::AccountKey => "account_key",
    }
}

impl From<RuntimeValidationError> for ProgramError {
    fn from(error: RuntimeValidationError) -> Self {
        match error {
            RuntimeValidationError::AccountCountMismatch { .. } => {
                ProgramError::NotEnoughAccountKeys
            }
            RuntimeValidationError::MissingRequiredSignature { .. } => {
                ProgramError::MissingRequiredSignature
            }
            RuntimeValidationError::AccountOwnerMismatch { .. }
            | RuntimeValidationError::IncorrectSystemProgramAccount { .. } => {
                ProgramError::IncorrectProgramId
            }
            RuntimeValidationError::PdaDerivationFailed { .. }
            | RuntimeValidationError::PdaMismatch { .. } => ProgramError::InvalidSeeds,
            RuntimeValidationError::UnsupportedTokenProgramOwnerValidation { .. }
            | RuntimeValidationError::UnknownPdaDescriptor { .. }
            | RuntimeValidationError::UnknownPdaSeedAccount { .. }
            | RuntimeValidationError::UnknownPdaSeedArg { .. }
            | RuntimeValidationError::MissingAccountPubkeyField { .. }
            | RuntimeValidationError::UnsupportedPdaSeedKind { .. }
            | RuntimeValidationError::UnknownPdaBumpArg { .. }
            | RuntimeValidationError::UnsupportedPdaBumpKind { .. }
            | RuntimeValidationError::InvalidPdaBumpArgType { .. } => ProgramError::InvalidArgument,
            RuntimeValidationError::AccountNotMutable { .. }
            | RuntimeValidationError::InitWithoutMutability { .. }
            | RuntimeValidationError::InitWithoutPayer { .. }
            | RuntimeValidationError::UnknownPayer { .. }
            | RuntimeValidationError::PayerMustBeSigner { .. }
            | RuntimeValidationError::InitWithoutSpace { .. }
            | RuntimeValidationError::CloseWithoutMutability { .. }
            | RuntimeValidationError::UnknownCloseTarget { .. }
            | RuntimeValidationError::CloseTargetMustBeMutable { .. }
            | RuntimeValidationError::InitAndCloseConflict { .. }
            | RuntimeValidationError::UnknownHasOneAccount { .. }
            | RuntimeValidationError::HasOneMismatch { .. } => ProgramError::InvalidAccountData,
        }
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey as SolanaPubkey;

    use crate::{
        AccountOwner, HasOneConstraintDescriptor, InstructionAccountConstraintDescriptor,
        InstructionAccountDescriptor, InstructionAccountPubkeyField,
        InstructionAccountRoleDescriptor, InstructionArg, InstructionArgValue,
        InstructionValidationContext, PdaBumpDescriptor, PdaBumpKindDescriptor, PdaDescriptor,
        PdaSeedDescriptor, PdaSeedKindDescriptor,
    };

    use super::*;

    #[test]
    fn validates_initialized_program_owned_account() {
        let accounts = [
            InstructionAccountDescriptor {
                name: "payer",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: false,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "escrow",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Escrow"),
                owner: Some(AccountOwner::Program),
                space: Some(128),
                is_mut: true,
                is_signer: false,
                pda: Some("escrow_pda"),
                constraints: Some(InstructionAccountConstraintDescriptor {
                    init: true,
                    payer: Some("payer"),
                    close_to: None,
                    rent_exempt: true,
                    has_one: &[],
                }),
            },
        ];

        assert_eq!(validate_instruction_accounts(&accounts), Ok(()));
    }

    #[test]
    fn rejects_missing_space_for_program_owned_init_account() {
        let accounts = [
            InstructionAccountDescriptor {
                name: "payer",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: false,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "escrow",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Escrow"),
                owner: Some(AccountOwner::Program),
                space: None,
                is_mut: true,
                is_signer: false,
                pda: Some("escrow_pda"),
                constraints: Some(InstructionAccountConstraintDescriptor {
                    init: true,
                    payer: Some("payer"),
                    close_to: None,
                    rent_exempt: true,
                    has_one: &[],
                }),
            },
        ];

        assert_eq!(
            validate_instruction_accounts(&accounts),
            Err(RuntimeValidationError::InitWithoutSpace { account: "escrow" })
        );
    }

    #[test]
    fn rejects_unknown_has_one_account_reference() {
        let accounts = [InstructionAccountDescriptor {
            name: "escrow",
            role: InstructionAccountRoleDescriptor::Account,
            account_type: Some("Escrow"),
            owner: Some(AccountOwner::Program),
            space: Some(128),
            is_mut: true,
            is_signer: false,
            pda: Some("escrow_pda"),
            constraints: Some(InstructionAccountConstraintDescriptor {
                init: false,
                payer: None,
                close_to: None,
                rent_exempt: false,
                has_one: &[HasOneConstraintDescriptor {
                    field: "depositor",
                    account: "depositor",
                }],
            }),
        }];

        assert_eq!(
            validate_instruction_accounts(&accounts),
            Err(RuntimeValidationError::UnknownHasOneAccount {
                account: "escrow",
                related_account: "depositor",
            })
        );
    }

    #[test]
    fn validates_runtime_has_one_with_pubkey_field_context() {
        let program_id = SolanaPubkey::new_unique();
        let depositor_key = SolanaPubkey::new_unique();
        let beneficiary_key = SolanaPubkey::new_unique();
        let escrow_key = SolanaPubkey::new_unique();
        let mut depositor_lamports = 1;
        let mut beneficiary_lamports = 1;
        let mut escrow_lamports = 1;
        let mut depositor_data = [];
        let mut beneficiary_data = [];
        let mut escrow_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "depositor",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "beneficiary",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: false,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "escrow",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Escrow"),
                owner: Some(AccountOwner::Program),
                space: Some(128),
                is_mut: true,
                is_signer: false,
                pda: None,
                constraints: Some(InstructionAccountConstraintDescriptor {
                    init: false,
                    payer: None,
                    close_to: None,
                    rent_exempt: false,
                    has_one: &[
                        HasOneConstraintDescriptor {
                            field: "depositor",
                            account: "depositor",
                        },
                        HasOneConstraintDescriptor {
                            field: "beneficiary",
                            account: "beneficiary",
                        },
                    ],
                }),
            },
        ];
        let actual = [
            AccountInfo::new(
                &depositor_key,
                true,
                true,
                &mut depositor_lamports,
                &mut depositor_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &beneficiary_key,
                false,
                true,
                &mut beneficiary_lamports,
                &mut beneficiary_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &escrow_key,
                false,
                true,
                &mut escrow_lamports,
                &mut escrow_data,
                &program_id,
                false,
            ),
        ];
        let context = InstructionValidationContext {
            args: &[],
            account_pubkey_fields: &[
                InstructionAccountPubkeyField {
                    account: "escrow",
                    field: "depositor",
                    value: depositor_key.to_bytes(),
                },
                InstructionAccountPubkeyField {
                    account: "escrow",
                    field: "beneficiary",
                    value: beneficiary_key.to_bytes(),
                },
            ],
        };

        assert_eq!(
            validate_program_account_infos_with_context(
                &program_id,
                &accounts,
                &actual,
                &[],
                &context
            ),
            Ok(())
        );
    }

    #[test]
    fn rejects_runtime_has_one_mismatch() {
        let program_id = SolanaPubkey::new_unique();
        let depositor_key = SolanaPubkey::new_unique();
        let beneficiary_key = SolanaPubkey::new_unique();
        let escrow_key = SolanaPubkey::new_unique();
        let wrong_beneficiary = SolanaPubkey::new_unique();
        let mut depositor_lamports = 1;
        let mut beneficiary_lamports = 1;
        let mut escrow_lamports = 1;
        let mut depositor_data = [];
        let mut beneficiary_data = [];
        let mut escrow_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "depositor",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "beneficiary",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: false,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "escrow",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Escrow"),
                owner: Some(AccountOwner::Program),
                space: Some(128),
                is_mut: true,
                is_signer: false,
                pda: None,
                constraints: Some(InstructionAccountConstraintDescriptor {
                    init: false,
                    payer: None,
                    close_to: None,
                    rent_exempt: false,
                    has_one: &[HasOneConstraintDescriptor {
                        field: "beneficiary",
                        account: "beneficiary",
                    }],
                }),
            },
        ];
        let actual = [
            AccountInfo::new(
                &depositor_key,
                true,
                true,
                &mut depositor_lamports,
                &mut depositor_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &beneficiary_key,
                false,
                true,
                &mut beneficiary_lamports,
                &mut beneficiary_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &escrow_key,
                false,
                true,
                &mut escrow_lamports,
                &mut escrow_data,
                &program_id,
                false,
            ),
        ];
        let context = InstructionValidationContext {
            args: &[],
            account_pubkey_fields: &[InstructionAccountPubkeyField {
                account: "escrow",
                field: "beneficiary",
                value: wrong_beneficiary.to_bytes(),
            }],
        };

        assert_eq!(
            validate_program_account_infos_with_context(
                &program_id,
                &accounts,
                &actual,
                &[],
                &context
            ),
            Err(RuntimeValidationError::HasOneMismatch {
                account: "escrow",
                field: "beneficiary",
                related_account: "beneficiary",
                expected_key: beneficiary_key,
                actual_key: wrong_beneficiary,
            })
        );
    }

    #[test]
    fn rejects_non_mutable_close_target() {
        let accounts = [
            InstructionAccountDescriptor {
                name: "recipient",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: false,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "escrow",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Escrow"),
                owner: Some(AccountOwner::Program),
                space: Some(128),
                is_mut: true,
                is_signer: false,
                pda: Some("escrow_pda"),
                constraints: Some(InstructionAccountConstraintDescriptor {
                    init: false,
                    payer: None,
                    close_to: Some("recipient"),
                    rent_exempt: false,
                    has_one: &[],
                }),
            },
        ];

        assert_eq!(
            validate_instruction_accounts(&accounts),
            Err(RuntimeValidationError::CloseTargetMustBeMutable {
                account: "escrow",
                target: "recipient",
            })
        );
    }

    #[test]
    fn rejects_init_and_close_conflict() {
        let accounts = [
            InstructionAccountDescriptor {
                name: "payer",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "escrow",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Escrow"),
                owner: Some(AccountOwner::Program),
                space: Some(128),
                is_mut: true,
                is_signer: false,
                pda: Some("escrow_pda"),
                constraints: Some(InstructionAccountConstraintDescriptor {
                    init: true,
                    payer: Some("payer"),
                    close_to: Some("payer"),
                    rent_exempt: false,
                    has_one: &[],
                }),
            },
        ];

        assert_eq!(
            validate_instruction_accounts(&accounts),
            Err(RuntimeValidationError::InitAndCloseConflict { account: "escrow" })
        );
    }

    #[test]
    fn maps_owner_mismatch_to_incorrect_program_id() {
        assert_eq!(
            ProgramError::from(RuntimeValidationError::AccountOwnerMismatch {
                account: "vault",
                expected_owner: SolanaPubkey::new_from_array([1; 32]),
                actual_owner: SolanaPubkey::new_from_array([2; 32]),
            }),
            ProgramError::IncorrectProgramId
        );
    }

    #[test]
    fn maps_pda_mismatch_to_invalid_seeds() {
        assert_eq!(
            ProgramError::from(RuntimeValidationError::PdaMismatch {
                account: "vault",
                pda: "vault_pda",
                expected_key: SolanaPubkey::new_from_array([1; 32]),
                actual_key: SolanaPubkey::new_from_array([2; 32]),
            }),
            ProgramError::InvalidSeeds
        );
    }

    #[test]
    fn rejects_unknown_pda_descriptor_reference() {
        let accounts = [InstructionAccountDescriptor {
            name: "vault",
            role: InstructionAccountRoleDescriptor::Account,
            account_type: Some("Vault"),
            owner: Some(AccountOwner::Program),
            space: Some(8),
            is_mut: true,
            is_signer: false,
            pda: Some("vault_pda"),
            constraints: None,
        }];

        let key = SolanaPubkey::new_unique();
        let owner = SolanaPubkey::new_unique();
        let mut lamports = 1;
        let mut data = [0_u8; 8];
        let actual = [AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
        )];

        assert_eq!(
            validate_program_account_infos_with_pdas(&owner, &accounts, &actual, &[]),
            Err(RuntimeValidationError::UnknownPdaDescriptor {
                account: "vault",
                pda: "vault_pda",
            })
        );
    }

    #[test]
    fn rejects_missing_arg_seed_context() {
        let program_id = SolanaPubkey::new_unique();
        let authority_key = SolanaPubkey::new_unique();
        let vault_key = SolanaPubkey::new_unique();
        let mut authority_lamports = 1;
        let mut vault_lamports = 1;
        let mut authority_data = [];
        let mut vault_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "authority",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "vault",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Vault"),
                owner: Some(AccountOwner::Program),
                space: Some(8),
                is_mut: true,
                is_signer: false,
                pda: Some("vault_pda"),
                constraints: None,
            },
        ];
        let actual = [
            AccountInfo::new(
                &authority_key,
                true,
                true,
                &mut authority_lamports,
                &mut authority_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &vault_key,
                false,
                true,
                &mut vault_lamports,
                &mut vault_data,
                &program_id,
                false,
            ),
        ];
        let pdas = [PdaDescriptor {
            name: "vault_pda",
            seeds: &[PdaSeedDescriptor {
                kind: PdaSeedKindDescriptor::Arg,
                value: "authority_bump",
            }],
            bump: Some(PdaBumpDescriptor {
                kind: PdaBumpKindDescriptor::Canonical,
                value: None,
            }),
        }];

        assert_eq!(
            validate_program_account_infos_with_pdas(&program_id, &accounts, &actual, &pdas),
            Err(RuntimeValidationError::UnknownPdaSeedArg {
                account: "vault",
                pda: "vault_pda",
                arg: "authority_bump",
            })
        );
    }

    #[test]
    fn validates_arg_seed_with_instruction_args() {
        let program_id = SolanaPubkey::new_unique();
        let authority_key = SolanaPubkey::new_unique();
        let label = "vault";
        let (vault_key, _bump) = SolanaPubkey::find_program_address(
            &[label.as_bytes(), authority_key.as_ref()],
            &program_id,
        );
        let mut authority_lamports = 1;
        let mut vault_lamports = 1;
        let mut authority_data = [];
        let mut vault_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "authority",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "vault",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Vault"),
                owner: Some(AccountOwner::Program),
                space: Some(8),
                is_mut: true,
                is_signer: false,
                pda: Some("vault_pda"),
                constraints: None,
            },
        ];
        let actual = [
            AccountInfo::new(
                &authority_key,
                true,
                true,
                &mut authority_lamports,
                &mut authority_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &vault_key,
                false,
                true,
                &mut vault_lamports,
                &mut vault_data,
                &program_id,
                false,
            ),
        ];
        let pdas = [PdaDescriptor {
            name: "vault_pda",
            seeds: &[
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::Arg,
                    value: "label",
                },
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::AccountKey,
                    value: "authority",
                },
            ],
            bump: Some(PdaBumpDescriptor {
                kind: PdaBumpKindDescriptor::Canonical,
                value: None,
            }),
        }];
        let instruction_args = [InstructionArg {
            name: "label",
            value: InstructionArgValue::String(label),
        }];

        assert_eq!(
            validate_program_account_infos_with_pdas_and_args(
                &program_id,
                &accounts,
                &actual,
                &pdas,
                &instruction_args,
            ),
            Ok(())
        );
    }

    #[test]
    fn validates_arg_bump_with_instruction_args() {
        let program_id = SolanaPubkey::new_unique();
        let authority_key = SolanaPubkey::new_unique();
        let (vault_key, bump) =
            SolanaPubkey::find_program_address(&[b"vault", authority_key.as_ref()], &program_id);
        let mut authority_lamports = 1;
        let mut vault_lamports = 1;
        let mut authority_data = [];
        let mut vault_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "authority",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "vault",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Vault"),
                owner: Some(AccountOwner::Program),
                space: Some(8),
                is_mut: true,
                is_signer: false,
                pda: Some("vault_pda"),
                constraints: None,
            },
        ];
        let actual = [
            AccountInfo::new(
                &authority_key,
                true,
                true,
                &mut authority_lamports,
                &mut authority_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &vault_key,
                false,
                true,
                &mut vault_lamports,
                &mut vault_data,
                &program_id,
                false,
            ),
        ];
        let pdas = [PdaDescriptor {
            name: "vault_pda",
            seeds: &[
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::Const,
                    value: "vault",
                },
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::AccountKey,
                    value: "authority",
                },
            ],
            bump: Some(PdaBumpDescriptor {
                kind: PdaBumpKindDescriptor::Arg,
                value: Some("vault_bump"),
            }),
        }];
        let instruction_args = [InstructionArg {
            name: "vault_bump",
            value: InstructionArgValue::U8(bump),
        }];

        assert_eq!(
            validate_program_account_infos_with_pdas_and_args(
                &program_id,
                &accounts,
                &actual,
                &pdas,
                &instruction_args,
            ),
            Ok(())
        );
    }

    #[test]
    fn rejects_missing_arg_seed_at_runtime() {
        let program_id = SolanaPubkey::new_unique();
        let authority_key = SolanaPubkey::new_unique();
        let vault_key = SolanaPubkey::new_unique();
        let mut authority_lamports = 1;
        let mut vault_lamports = 1;
        let mut authority_data = [];
        let mut vault_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "authority",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "vault",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Vault"),
                owner: Some(AccountOwner::Program),
                space: Some(8),
                is_mut: true,
                is_signer: false,
                pda: Some("vault_pda"),
                constraints: None,
            },
        ];
        let actual = [
            AccountInfo::new(
                &authority_key,
                true,
                true,
                &mut authority_lamports,
                &mut authority_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &vault_key,
                false,
                true,
                &mut vault_lamports,
                &mut vault_data,
                &program_id,
                false,
            ),
        ];
        let pdas = [PdaDescriptor {
            name: "vault_pda",
            seeds: &[PdaSeedDescriptor {
                kind: PdaSeedKindDescriptor::Arg,
                value: "label",
            }],
            bump: Some(PdaBumpDescriptor {
                kind: PdaBumpKindDescriptor::Canonical,
                value: None,
            }),
        }];

        assert_eq!(
            validate_program_account_infos_with_pdas_and_args(
                &program_id,
                &accounts,
                &actual,
                &pdas,
                &[],
            ),
            Err(RuntimeValidationError::UnknownPdaSeedArg {
                account: "vault",
                pda: "vault_pda",
                arg: "label",
            })
        );
    }

    #[test]
    fn rejects_non_u8_arg_bump_type() {
        let program_id = SolanaPubkey::new_unique();
        let authority_key = SolanaPubkey::new_unique();
        let vault_key = SolanaPubkey::new_unique();
        let mut authority_lamports = 1;
        let mut vault_lamports = 1;
        let mut authority_data = [];
        let mut vault_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "authority",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: true,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "vault",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Vault"),
                owner: Some(AccountOwner::Program),
                space: Some(8),
                is_mut: true,
                is_signer: false,
                pda: Some("vault_pda"),
                constraints: None,
            },
        ];
        let actual = [
            AccountInfo::new(
                &authority_key,
                true,
                true,
                &mut authority_lamports,
                &mut authority_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &vault_key,
                false,
                true,
                &mut vault_lamports,
                &mut vault_data,
                &program_id,
                false,
            ),
        ];
        let pdas = [PdaDescriptor {
            name: "vault_pda",
            seeds: &[
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::Const,
                    value: "vault",
                },
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::AccountKey,
                    value: "authority",
                },
            ],
            bump: Some(PdaBumpDescriptor {
                kind: PdaBumpKindDescriptor::Arg,
                value: Some("vault_bump"),
            }),
        }];
        let instruction_args = [InstructionArg {
            name: "vault_bump",
            value: InstructionArgValue::U64(7),
        }];

        assert_eq!(
            validate_program_account_infos_with_pdas_and_args(
                &program_id,
                &accounts,
                &actual,
                &pdas,
                &instruction_args,
            ),
            Err(RuntimeValidationError::InvalidPdaBumpArgType {
                account: "vault",
                pda: "vault_pda",
                arg: "vault_bump",
                kind: "u64",
            })
        );
    }

    #[test]
    fn validates_account_field_seed_with_pubkey_field_context() {
        let program_id = SolanaPubkey::new_unique();
        let authority_key = SolanaPubkey::new_unique();
        let profile_key = SolanaPubkey::new_unique();
        let profile_authority = SolanaPubkey::new_unique();
        let (vault_key, _bump) = SolanaPubkey::find_program_address(
            &[b"vault", profile_authority.as_ref()],
            &program_id,
        );
        let mut authority_lamports = 1;
        let mut profile_lamports = 1;
        let mut vault_lamports = 1;
        let mut authority_data = [];
        let mut profile_data = [0_u8; 64];
        let mut vault_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "authority",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: false,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "profile",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Profile"),
                owner: Some(AccountOwner::Program),
                space: Some(64),
                is_mut: false,
                is_signer: false,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "vault",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Vault"),
                owner: Some(AccountOwner::Program),
                space: Some(8),
                is_mut: true,
                is_signer: false,
                pda: Some("vault_pda"),
                constraints: None,
            },
        ];
        let actual = [
            AccountInfo::new(
                &authority_key,
                true,
                false,
                &mut authority_lamports,
                &mut authority_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &profile_key,
                false,
                false,
                &mut profile_lamports,
                &mut profile_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &vault_key,
                false,
                true,
                &mut vault_lamports,
                &mut vault_data,
                &program_id,
                false,
            ),
        ];
        let pdas = [PdaDescriptor {
            name: "vault_pda",
            seeds: &[
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::Const,
                    value: "vault",
                },
                PdaSeedDescriptor {
                    kind: PdaSeedKindDescriptor::AccountField,
                    value: "profile.authority",
                },
            ],
            bump: Some(PdaBumpDescriptor {
                kind: PdaBumpKindDescriptor::Canonical,
                value: None,
            }),
        }];
        let context = InstructionValidationContext {
            args: &[],
            account_pubkey_fields: &[InstructionAccountPubkeyField {
                account: "profile",
                field: "authority",
                value: profile_authority.to_bytes(),
            }],
        };

        assert_eq!(
            validate_program_account_infos_with_context(
                &program_id,
                &accounts,
                &actual,
                &pdas,
                &context,
            ),
            Ok(())
        );
    }

    #[test]
    fn rejects_missing_account_field_seed_context() {
        let program_id = SolanaPubkey::new_unique();
        let authority_key = SolanaPubkey::new_unique();
        let profile_key = SolanaPubkey::new_unique();
        let vault_key = SolanaPubkey::new_unique();
        let mut authority_lamports = 1;
        let mut profile_lamports = 1;
        let mut vault_lamports = 1;
        let mut authority_data = [];
        let mut profile_data = [0_u8; 64];
        let mut vault_data = [0_u8; 8];
        let accounts = [
            InstructionAccountDescriptor {
                name: "authority",
                role: InstructionAccountRoleDescriptor::Signer,
                account_type: None,
                owner: None,
                space: None,
                is_mut: false,
                is_signer: true,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "profile",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Profile"),
                owner: Some(AccountOwner::Program),
                space: Some(64),
                is_mut: false,
                is_signer: false,
                pda: None,
                constraints: None,
            },
            InstructionAccountDescriptor {
                name: "vault",
                role: InstructionAccountRoleDescriptor::Account,
                account_type: Some("Vault"),
                owner: Some(AccountOwner::Program),
                space: Some(8),
                is_mut: true,
                is_signer: false,
                pda: Some("vault_pda"),
                constraints: None,
            },
        ];
        let actual = [
            AccountInfo::new(
                &authority_key,
                true,
                false,
                &mut authority_lamports,
                &mut authority_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &profile_key,
                false,
                false,
                &mut profile_lamports,
                &mut profile_data,
                &program_id,
                false,
            ),
            AccountInfo::new(
                &vault_key,
                false,
                true,
                &mut vault_lamports,
                &mut vault_data,
                &program_id,
                false,
            ),
        ];
        let pdas = [PdaDescriptor {
            name: "vault_pda",
            seeds: &[PdaSeedDescriptor {
                kind: PdaSeedKindDescriptor::AccountField,
                value: "profile.authority",
            }],
            bump: Some(PdaBumpDescriptor {
                kind: PdaBumpKindDescriptor::Canonical,
                value: None,
            }),
        }];

        assert_eq!(
            validate_program_account_infos_with_context(
                &program_id,
                &accounts,
                &actual,
                &pdas,
                &InstructionValidationContext::default(),
            ),
            Err(RuntimeValidationError::MissingAccountPubkeyField {
                account: "profile",
                field: "authority",
            })
        );
    }
}
