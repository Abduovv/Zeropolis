use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Cycle is not active")]
    CycleNotActive,
    #[msg("Cycle is full")]
    CycleFull,
    #[msg("Invalid payout order")]
    InvalidPayoutOrder,
    #[msg("Not in payout order")]
    NotInPayoutOrder,
    #[msg("Invalid cycle")]
    InvalidCycle,
    #[msg("Invalid member")]
    InvalidMember,
    #[msg("Member is not active")]
    MemberNotActive,
    #[msg("Contribution is late")]
    ContributionLate,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,
    #[msg("Payout too early")]
    PayoutTooEarly,
    #[msg("Cycle is complete")]
    CycleComplete,
    #[msg("Invalid payout recipient")]
    InvalidPayoutRecipient,
    #[msg("Cycle has already started")]
    CycleAlreadyStarted,
    #[msg("Too early to report default")]
    TooEarlyToReport,
    #[msg("Member has not defaulted")]
    MemberNotDefaulted,
    #[msg("Member is still active")]
    MemberStillActive,
    #[msg("Too many cycles")]
    TooManyCycles,
    #[msg("Invalid stake amount")]
    InvalidStakeAmount,
    #[msg("Cycle is still active")]
    CycleStillActive,
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("Insufficient stake amount")]
    InsufficientStake,
}