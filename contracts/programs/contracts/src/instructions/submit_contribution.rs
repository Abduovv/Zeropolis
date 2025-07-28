use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct SubmitContribution<'info> {
    #[account(mut)]
    pub member: Signer<'info>,

    #[account(
        mut,
        has_one = organizer,
        constraint = cycle.is_active @ CustomError::CycleNotActive
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        mut,
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = member_account.member == member.key() @ CustomError::InvalidMember,
        constraint = member_account.is_active @ CustomError::MemberNotActive
    )]
    pub member_account: Account<'info, MemberAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = cycle
    )]
    pub cycle_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = member
    )]
    pub member_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub organizer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> SubmitContribution<'info> {
    pub fn submit_contribution(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp <= self.cycle.next_round_time,
            CustomError::ContributionLate
        );

        // Transfer contribution to cycle token account
        let contribution_amount = self.cycle.amount_per_user;
        let cpi_accounts = Transfer {
            from: self.member_token_account.to_account_info(),
            to: self.cycle_token_account.to_account_info(),
            authority: self.member.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::transfer(cpi_ctx, contribution_amount)?;

        // Update member account
        self.member_account.contributions_made = self.member_account.contributions_made
            .checked_add(1)
            .ok_or(CustomError::ArithmeticOverflow)?;

        Ok(())
    }
}