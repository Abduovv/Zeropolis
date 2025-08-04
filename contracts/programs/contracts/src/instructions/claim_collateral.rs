use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, TokenAccount, Transfer},
};

use crate::{error::CustomError, CycleAccount, MemberAccount};

#[derive(Accounts)]
pub struct ClaimCollateral<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,
    
    #[account(
        mut,
        has_one = organizer,
        constraint = cycle.is_active @ CustomError::CycleNotActive,
        seeds = [b"cycle", organizer.key().as_ref(), cycle.nonces.to_le_bytes().as_ref()],
        bump = cycle.bump
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        mut,
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = !member_account.is_active @ CustomError::MemberStillActive,
        has_one = member,
        close = member
    )]
    pub member_account: Account<'info, MemberAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = claimer
    )]
    pub claimer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = cycle
    )]
    pub cycle_token_account: Account<'info, TokenAccount>,

    pub member: SystemAccount<'info>,
    pub organizer: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimCollateral<'info> {
    pub fn claim_collateral(&mut self) -> Result<()> {
    let signer_seeds = &[
    b"cycle",
    self.organizer.key.as_ref(),
    &self.cycle.nonces.to_le_bytes(), 
    &[self.cycle.bump],
];

    let signer = &[&signer_seeds[..]];

    let cpi_accounts = Transfer {
        from: self.cycle_token_account.to_account_info(),
        to: self.claimer_token_account.to_account_info(),
        authority: self.cycle.to_account_info(),
    };

    let cpi_program = self.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

    if self.claimer.key() == self.organizer.key() {
        anchor_spl::token::transfer(cpi_ctx, self.cycle.organizer_stake)?;
    } else if self.claimer.key() == self.member.key() {
        anchor_spl::token::transfer(cpi_ctx, self.member_account.collateral)?;
    } else {
        return err!(CustomError::UnauthorizedClaimer);
    }

    Ok(())
}
}
