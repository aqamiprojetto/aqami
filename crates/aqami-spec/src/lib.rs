mod inspect;
mod load;
mod model;
mod normalize;
mod validate;

pub use inspect::ProjectInspection;
pub use load::{LoadedProjectSpec, SpecLoadError, load_project_spec};
pub use model::{
    AccountOwner, AccountSpec, AqamiProjectSpec, Cluster, EventSpec, FieldSpec, FrameworkErrorSpec,
    InstructionAccountConstraintsSpec, InstructionAccountRole, InstructionAccountSpec,
    InstructionSpec, PackageSpec, PdaSpec, ProgramSpec, SeedKind, SeedSpec,
};
pub use normalize::{
    NormalizedAccount, NormalizedAccountOwner, NormalizedError, NormalizedEvent, NormalizedField,
    NormalizedInstruction, NormalizedInstructionAccount, NormalizedInstructionAccountConstraints,
    NormalizedPackage, NormalizedPda, NormalizedProgram, NormalizedProjectSpec,
    normalization_diagnostics, normalize_project_spec, rust_type_name,
};
pub use validate::{Diagnostic, ValidationOutcome, validate_project_spec};

pub const PROJECT_SCHEMA_JSON: &str = include_str!("../../../schemas/aqami.project.schema.json");
