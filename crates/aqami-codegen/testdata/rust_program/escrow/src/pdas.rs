use aqami_runtime::{PdaBumpDescriptor, PdaBumpKindDescriptor, PdaDescriptor, PdaSeedDescriptor, PdaSeedKindDescriptor};

/// Deterministic escrow PDA derived from the depositor and beneficiary.
pub const ESCROW_PDA: &str = "escrow_pda";
pub const ESCROW_PDA_DESCRIPTOR: PdaDescriptor = PdaDescriptor { name: "escrow_pda", seeds: &[PdaSeedDescriptor { kind: PdaSeedKindDescriptor::Const, value: "escrow" }, PdaSeedDescriptor { kind: PdaSeedKindDescriptor::AccountKey, value: "depositor" }, PdaSeedDescriptor { kind: PdaSeedKindDescriptor::AccountKey, value: "beneficiary" }], bump: Some(PdaBumpDescriptor { kind: PdaBumpKindDescriptor::Canonical, value: None }) };

pub fn escrow_pda_seed_descriptions() -> &'static [&'static str] {
    &[
        "const: escrow",
        "account_key: depositor",
        "account_key: beneficiary",
    ]
}
pub fn escrow_pda_bump_description() -> Option<&'static str> {
    Some("canonical")
}

