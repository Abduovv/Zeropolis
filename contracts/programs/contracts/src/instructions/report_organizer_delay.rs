use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct ReportOrganizerDelay<'info> {
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
        associated_token::mint = cycle.token_mint,
        associated_token::authority = cycle
    )]
    pub cycle_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub organizer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ReportOrganizerDelay<'info> {
    pub fn report_organizer_delay(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        let grace_period = 86_400; // 1 day in seconds
        require!(
            clock.unix_timestamp > self.cycle.next_round_time + grace_period,
            CustomError::TooEarlyToReport
        );

        // Calculate days delayed
        let seconds_delayed = clock.unix_timestamp - self.cycle.next_round_time;
        let days_delayed = seconds_delayed / 86_400;
        let penalty_percentage = days_delayed * 10; // 10% per day
        let penalty_amount = if penalty_percentage >= 100 {
            self.cycle.organizer_stake
        } else {
            self.cycle.organizer_stake
                .checked_mul(penalty_percentage as u64)
                .ok_or(CustomError::ArithmeticOverflow)?
                / 100
        };

        // Update organizer stake
        self.cycle.organizer_stake = self.cycle.organizer_stake
            .checked_sub(penalty_amount)
            .ok_or(CustomError::ArithmeticUnderflow)?;

        // Update slashed stakes for redistribution
        self.cycle.slashed_stakes = self.cycle.slashed_stakes
            .checked_add(penalty_amount)
            .ok_or(CustomError::ArithmeticOverflow)?;

        // Deactivate cycle if organizer stake is fully slashed
        if self.cycle.organizer_stake == 0 {
            self.cycle.is_active = false;
        }

        Ok(())
    }
}