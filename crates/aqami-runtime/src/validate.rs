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
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        AccountOwner, InstructionAccountConstraintDescriptor, InstructionAccountDescriptor,
        InstructionAccountRoleDescriptor,
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
                }),
            },
        ];

        assert_eq!(
            validate_instruction_accounts(&accounts),
            Err(RuntimeValidationError::InitWithoutSpace { account: "escrow" })
        );
    }
}
