use thiserror::Error;

use crate::{AccountOwner, InstructionAccountDescriptor};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuntimeValidationError {
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

#[cfg(test)]
mod tests {
    use crate::{
        AccountOwner, HasOneConstraintDescriptor, InstructionAccountConstraintDescriptor,
        InstructionAccountDescriptor, InstructionAccountRoleDescriptor,
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
}
