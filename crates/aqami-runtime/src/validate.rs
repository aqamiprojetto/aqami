use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use solana_system_interface::program as system_program;
use thiserror::Error;

use crate::{
    AccountOwner, InstructionAccountDescriptor, InstructionAccountRoleDescriptor,
    PdaBumpKindDescriptor, PdaDescriptor, PdaSeedKindDescriptor,
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
        expected_owner: Pubkey,
        actual_owner: Pubkey,
    },
    #[error("runtime account `{account}` must be the system program, got `{actual_key}`")]
    IncorrectSystemProgramAccount {
        account: &'static str,
        actual_key: Pubkey,
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
    #[error("PDA `{pda}` for account `{account}` uses unsupported seed kind `{kind}`")]
    UnsupportedPdaSeedKind {
        account: &'static str,
        pda: &'static str,
        kind: &'static str,
    },
    #[error("PDA `{pda}` for account `{account}` uses unsupported bump kind `{kind}`")]
    UnsupportedPdaBumpKind {
        account: &'static str,
        pda: &'static str,
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
        expected_key: Pubkey,
        actual_key: Pubkey,
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
    program_id: &Pubkey,
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

/// Validates program-context account metadata plus currently supported PDA semantics.
///
/// This extends `validate_program_account_infos` with PDA derivation checks for
/// AQAMI's currently resolvable seed and bump forms:
///
/// - `const` seeds
/// - `account_key` seeds
/// - canonical bumps
///
/// AQAMI intentionally returns explicit errors for unresolved PDA features such
/// as `arg` seeds, `account_field` seeds, and arg-backed bumps until the runtime
/// exposes the additional execution context required to validate them safely.
pub fn validate_program_account_infos_with_pdas(
    program_id: &Pubkey,
    expected: &[InstructionAccountDescriptor],
    actual: &[AccountInfo<'_>],
    pda_descriptors: &[PdaDescriptor],
) -> Result<(), RuntimeValidationError> {
    validate_program_account_infos(program_id, expected, actual)?;

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

        let mut seed_slices = Vec::with_capacity(pda_descriptor.seeds.len());
        for seed in pda_descriptor.seeds {
            match seed.kind {
                PdaSeedKindDescriptor::Const => seed_slices.push(seed.value.as_bytes()),
                PdaSeedKindDescriptor::AccountKey => {
                    let Some(seed_account_index) = instruction_account_index(expected, seed.value)
                    else {
                        return Err(RuntimeValidationError::UnknownPdaSeedAccount {
                            account: descriptor.name,
                            pda: pda_name,
                            seed_account: seed.value,
                        });
                    };
                    seed_slices.push(actual[seed_account_index].key.as_ref());
                }
                PdaSeedKindDescriptor::Arg => {
                    return Err(RuntimeValidationError::UnsupportedPdaSeedKind {
                        account: descriptor.name,
                        pda: pda_name,
                        kind: pda_seed_kind_name(seed.kind),
                    });
                }
                PdaSeedKindDescriptor::AccountField => {
                    return Err(RuntimeValidationError::UnsupportedPdaSeedKind {
                        account: descriptor.name,
                        pda: pda_name,
                        kind: pda_seed_kind_name(seed.kind),
                    });
                }
            }
        }

        let expected_key = match pda_descriptor.bump {
            None => Pubkey::create_program_address(&seed_slices, program_id).map_err(|_| {
                RuntimeValidationError::PdaDerivationFailed {
                    account: descriptor.name,
                    pda: pda_name,
                }
            })?,
            Some(bump) => match bump.kind {
                PdaBumpKindDescriptor::Canonical => {
                    Pubkey::find_program_address(&seed_slices, program_id).0
                }
                PdaBumpKindDescriptor::Arg => {
                    return Err(RuntimeValidationError::UnsupportedPdaBumpKind {
                        account: descriptor.name,
                        pda: pda_name,
                        kind: pda_bump_kind_name(bump.kind),
                    });
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

fn instruction_account_index(
    expected: &[InstructionAccountDescriptor],
    account_name: &str,
) -> Option<usize> {
    expected
        .iter()
        .position(|candidate| candidate.name == account_name)
}

fn pda_seed_kind_name(kind: PdaSeedKindDescriptor) -> &'static str {
    match kind {
        PdaSeedKindDescriptor::Const => "const",
        PdaSeedKindDescriptor::Arg => "arg",
        PdaSeedKindDescriptor::AccountField => "account_field",
        PdaSeedKindDescriptor::AccountKey => "account_key",
    }
}

fn pda_bump_kind_name(kind: PdaBumpKindDescriptor) -> &'static str {
    match kind {
        PdaBumpKindDescriptor::Canonical => "canonical",
        PdaBumpKindDescriptor::Arg => "arg",
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
            | RuntimeValidationError::UnsupportedPdaSeedKind { .. }
            | RuntimeValidationError::UnsupportedPdaBumpKind { .. } => {
                ProgramError::InvalidArgument
            }
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
            | RuntimeValidationError::UnknownHasOneAccount { .. } => {
                ProgramError::InvalidAccountData
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AccountOwner, HasOneConstraintDescriptor, InstructionAccountConstraintDescriptor,
        InstructionAccountDescriptor, InstructionAccountRoleDescriptor, PdaBumpDescriptor,
        PdaBumpKindDescriptor, PdaDescriptor, PdaSeedDescriptor, PdaSeedKindDescriptor,
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
                expected_owner: Pubkey::new_from_array([1; 32]),
                actual_owner: Pubkey::new_from_array([2; 32]),
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
                expected_key: Pubkey::new_from_array([1; 32]),
                actual_key: Pubkey::new_from_array([2; 32]),
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

        let key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
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
    fn rejects_unsupported_arg_seed_kind() {
        let program_id = Pubkey::new_unique();
        let authority_key = Pubkey::new_unique();
        let vault_key = Pubkey::new_unique();
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
            Err(RuntimeValidationError::UnsupportedPdaSeedKind {
                account: "vault",
                pda: "vault_pda",
                kind: "arg",
            })
        );
    }
}
