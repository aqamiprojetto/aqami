#![allow(deprecated)]

use aqami_runtime::{
    AccountOwner, InstructionAccountConstraintDescriptor, InstructionAccountDescriptor,
    InstructionAccountRoleDescriptor, validate_program_account_infos,
};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use solana_program_test::{ProgramTest, processor};
use solana_signer::Signer;
use solana_system_interface::program as system_program;
use solana_transaction::Transaction;

const ACCOUNT_DESCRIPTORS: &[InstructionAccountDescriptor] = &[
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
        pda: None,
        constraints: Some(InstructionAccountConstraintDescriptor {
            init: false,
            payer: None,
            close_to: Some("authority"),
            rent_exempt: false,
            has_one: &[],
        }),
    },
    InstructionAccountDescriptor {
        name: "system_program",
        role: InstructionAccountRoleDescriptor::SystemProgram,
        account_type: None,
        owner: None,
        space: None,
        is_mut: false,
        is_signer: false,
        pda: None,
        constraints: None,
    },
];

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo<'_>],
    _instruction_data: &[u8],
) -> ProgramResult {
    validate_program_account_infos(program_id, ACCOUNT_DESCRIPTORS, accounts).map_err(Into::into)
}

fn build_instruction(
    program_id: Pubkey,
    authority: Pubkey,
    vault: Pubkey,
    system_program_key: Pubkey,
    authority_writable: bool,
    authority_signer: bool,
) -> Instruction {
    let authority_meta = if authority_writable {
        AccountMeta::new(authority, authority_signer)
    } else {
        AccountMeta::new_readonly(authority, authority_signer)
    };

    Instruction {
        program_id,
        accounts: vec![
            authority_meta,
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program_key, false),
        ],
        data: Vec::new(),
    }
}

fn test_account(lamports: u64, owner: Pubkey, data_len: usize) -> Account {
    Account {
        lamports,
        data: vec![0; data_len],
        owner,
        executable: false,
        rent_epoch: 0,
    }
}

#[tokio::test]
async fn accepts_matching_runtime_account_metas() {
    let program_id = Pubkey::new_unique();
    let authority = Keypair::new();
    let vault = Keypair::new();

    let mut program_test = ProgramTest::new(
        "aqami-runtime-test",
        program_id,
        processor!(process_instruction),
    );
    program_test.add_account(
        authority.pubkey(),
        test_account(1_000_000_000, Pubkey::new_unique(), 0),
    );
    program_test.add_account(vault.pubkey(), test_account(1_000_000_000, program_id, 8));

    let context = program_test.start_with_context().await;
    let instruction = build_instruction(
        program_id,
        authority.pubkey(),
        vault.pubkey(),
        system_program::ID,
        true,
        true,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .expect("matching signer and writable flags should pass");
}

#[tokio::test]
async fn rejects_missing_runtime_signature() {
    let program_id = Pubkey::new_unique();
    let authority = Keypair::new();
    let vault = Keypair::new();

    let mut program_test = ProgramTest::new(
        "aqami-runtime-test",
        program_id,
        processor!(process_instruction),
    );
    program_test.add_account(
        authority.pubkey(),
        test_account(1_000_000_000, Pubkey::new_unique(), 0),
    );
    program_test.add_account(vault.pubkey(), test_account(1_000_000_000, program_id, 8));

    let context = program_test.start_with_context().await;
    let instruction = build_instruction(
        program_id,
        authority.pubkey(),
        vault.pubkey(),
        system_program::ID,
        true,
        false,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("missing signer flag should fail");

    assert!(format!("{error:?}").contains("MissingRequiredSignature"));
}

#[tokio::test]
async fn rejects_read_only_close_target() {
    let program_id = Pubkey::new_unique();
    let authority = Keypair::new();
    let vault = Keypair::new();

    let mut program_test = ProgramTest::new(
        "aqami-runtime-test",
        program_id,
        processor!(process_instruction),
    );
    program_test.add_account(
        authority.pubkey(),
        test_account(1_000_000_000, Pubkey::new_unique(), 0),
    );
    program_test.add_account(vault.pubkey(), test_account(1_000_000_000, program_id, 8));

    let context = program_test.start_with_context().await;
    let instruction = build_instruction(
        program_id,
        authority.pubkey(),
        vault.pubkey(),
        system_program::ID,
        false,
        true,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("read-only close target should fail");

    assert!(format!("{error:?}").contains("InvalidAccountData"));
}

#[tokio::test]
async fn rejects_program_owned_account_with_wrong_owner() {
    let program_id = Pubkey::new_unique();
    let authority = Keypair::new();
    let vault = Keypair::new();

    let mut program_test = ProgramTest::new(
        "aqami-runtime-test",
        program_id,
        processor!(process_instruction),
    );
    program_test.add_account(
        authority.pubkey(),
        test_account(1_000_000_000, Pubkey::new_unique(), 0),
    );
    program_test.add_account(
        vault.pubkey(),
        test_account(1_000_000_000, Pubkey::new_unique(), 8),
    );

    let context = program_test.start_with_context().await;
    let instruction = build_instruction(
        program_id,
        authority.pubkey(),
        vault.pubkey(),
        system_program::ID,
        true,
        true,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("wrong program owner should fail");

    assert!(format!("{error:?}").contains("IncorrectProgramId"));
}

#[tokio::test]
async fn rejects_wrong_system_program_account() {
    let program_id = Pubkey::new_unique();
    let authority = Keypair::new();
    let vault = Keypair::new();
    let fake_system_program = Pubkey::new_unique();

    let mut program_test = ProgramTest::new(
        "aqami-runtime-test",
        program_id,
        processor!(process_instruction),
    );
    program_test.add_account(
        authority.pubkey(),
        test_account(1_000_000_000, Pubkey::new_unique(), 0),
    );
    program_test.add_account(vault.pubkey(), test_account(1_000_000_000, program_id, 8));
    program_test.add_account(
        fake_system_program,
        test_account(1_000_000_000, Pubkey::new_unique(), 0),
    );

    let context = program_test.start_with_context().await;
    let instruction = build_instruction(
        program_id,
        authority.pubkey(),
        vault.pubkey(),
        fake_system_program,
        true,
        true,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &authority],
        context.last_blockhash,
    );

    let error = context
        .banks_client
        .process_transaction(transaction)
        .await
        .expect_err("wrong system-program address should fail");

    assert!(format!("{error:?}").contains("IncorrectProgramId"));
}
