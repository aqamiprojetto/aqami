#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(i64)]
pub enum ProgramError {
    /// Escrow amount must be greater than zero.
    InvalidAmount = 1000,
    /// Escrow has already been released.
    EscrowAlreadyReleased = 1001,
}
