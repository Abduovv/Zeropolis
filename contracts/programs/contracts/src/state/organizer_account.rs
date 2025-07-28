use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct OrganizerAccount {
    pub total_cycles: u8,         // Total active cycles created by the organizer
    pub last_cycle_time: i64,    // Timestamp of the last cycle created
    pub locked_stake: u64,       // Total staked amount frozen across all cycles
    pub bump: u8,                // PDA bump seed
}