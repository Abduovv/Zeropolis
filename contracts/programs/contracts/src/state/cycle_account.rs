use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct CycleAccount {
    pub organizer: Pubkey,              // Organizer's public key
    pub token_mint: Pubkey,            // USDT mint address
    pub amount_per_user: u64,          // Contribution amount per user per round
    pub max_participants: u8,          // Maximum number of participants
    pub current_participants: u8,      // Current number of participants
    pub organizer_fee_bps: u16,        // Organizer fee in basis points
    pub is_active: bool,               // Whether the cycle is active
    pub contribution_interval: i64,    // Seconds between contribution rounds (e.g., 1 week)
    pub contributions_per_payout: u8,  // Number of contributions before a payout (e.g., 4)
    pub round_count: u8,               // Total payout rounds
    pub created_at: i64,               // Timestamp of cycle creation
    pub current_round: u8,             // Current contribution round
    pub next_round_time: i64,          // Unix timestamp of next contribution round
    #[max_len(200)]
    pub payout_order: Vec<Pubkey>,     // Order of payout recipients
    pub organizer_stake: u64,          // Organizer's staked amount for this cycle
    pub pot_amount: u64,               // Calculated pot amount per payout
    pub slashed_stakes: u64,           // Total slashed USDT for redistribution
    pub bump: u8,                      // PDA bump seed
}