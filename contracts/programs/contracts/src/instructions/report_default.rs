use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};
use crate::{error::CustomError, util::calculate_member_penalty, CycleAccount, MemberAccount};

#[derive(Accounts)]
pub struct ReportDefault<'info> {
    #[account(mut)]
    pub reporter: Signer<'info>,

    #[account(
        mut,
        has_one = organizer,
        constraint = cycle.is_active @ CustomError::CycleNotActive
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        mut,
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = member_account.is_active @ CustomError::MemberNotActive
    )]
    pub member_account: Account<'info, MemberAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = cycle
    )]
    pub cycle_token_account: Account<'info, TokenAccount>,
/// CHECK
    #[account(mut)]
    pub organizer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ReportDefault<'info> {
    pub fn report_default(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp > self.cycle.next_round_time,
            CustomError::TooEarlyToReport
        );

        // Calculate missed rounds
        let missed_rounds = (self.cycle.current_round + 1)
            .checked_sub(self.member_account.contributions_made)
            .ok_or(CustomError::ArithmeticUnderflow)?;

        // Calculate penalty using util function
        let penalty_amount = calculate_member_penalty(
            self.member_account.collateral,
            missed_rounds as u64,
            self.member_account.payout_received,
        )?;

        // Update member stake
        self.member_account.collateral = self.member_account.collateral
            .checked_sub(penalty_amount)
            .ok_or(CustomError::ArithmeticUnderflow)?;

        // Update slashed stakes for redistribution
        self.cycle.slashed_stakes = self.cycle.slashed_stakes
            .checked_add(penalty_amount)
            .ok_or(CustomError::ArithmeticOverflow)?;

        // Mark member inactive if fully slashed
        if self.member_account.collateral == 0 {
            self.member_account.is_active = false;
            self.cycle.payout_order.retain(|&x| x != self.member_account.member);
            self.cycle.current_participants = self.cycle.current_participants
                .checked_sub(1)
                .ok_or(CustomError::ArithmeticUnderflow)?;
        }

        Ok(())
    }
}