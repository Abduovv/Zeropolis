use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod util;


pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use util::*;

declare_id!("B1BXbY7CLKJW4TeJyFgNxzwRtLDEUuFMZTVWbv3FBE9s");

#[program]
pub mod money_cycle {
    use super::*;
    // Create a new cycle
    pub fn create_cycle(
        ctx: Context<CreateCycle>,
        args: CreateCycleArgs,
    ) -> Result<()> {
        ctx.accounts.create_cycle(args, ctx.bumps)
    }

    // Join an existing cycle
    pub fn join_cycle(ctx: Context<JoinCycle>) -> Result<()> {
        ctx.accounts.join_cycle(ctx.bumps)
    }

    // Submit a contribution
    pub fn submit_contribution(ctx: Context<SubmitContribution>) -> Result<()> {
        ctx.accounts.submit_contribution()
    }

    // Trigger a payout
    pub fn trigger_payout(ctx: Context<TriggerPayout>) -> Result<()> {
        ctx.accounts.trigger_payout()
    }

    // Exit a cycle (before it starts)
    pub fn exit_cycle(ctx: Context<ExitCycle>) -> Result<()> {
        ctx.accounts.exit_cycle()
    }

    // Report a member default
    pub fn report_default(ctx: Context<ReportDefault>) -> Result<()> {
        ctx.accounts.report_default()
    }

    // Report organizer delay
    pub fn report_organizer_delay(ctx: Context<ReportOrganizerDelay>) -> Result<()> {
        ctx.accounts.report_organizer_delay()
    }

    // Claim collateral from a defaulted member
    pub fn claim_collateral(ctx: Context<ClaimCollateral>) -> Result<()> {
        ctx.accounts.claim_collateral()
    }

    // Close a cycle
    pub fn close_cycle(ctx: Context<CloseCycle>) -> Result<()> {
        ctx.accounts.close_cycle()
    }
}