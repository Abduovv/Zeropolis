use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct MemberAccount {
    pub cycle: Pubkey,            // The cycle this member belongs to
    pub member: Pubkey,           // The member's public key
    pub contributions_made: u8,   // Number of contributions paid
    pub payout_received: bool,    // Whether the member has received their payout
    pub collateral: u64,          // Collateral staked by the member
    pub is_active: bool,          // Whether the member is still active
    pub bump: u8,                 // PDA bump seed
}