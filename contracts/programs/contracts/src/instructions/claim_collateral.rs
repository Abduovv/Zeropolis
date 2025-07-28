use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct ClaimCollateral<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        mut,
        has_one = organizer,
        constraint = cycle.is_active @ CustomError::CycleNotActive
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        mut,
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = !member_account.is_active @ CustomError::MemberStillActive,
        close = organizer
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
        associated_token::authority = organizer
    )]
    pub organizer_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimCollateral<'info> {
    pub fn claim_collateral(&mut self) -> Result<()> {
        // Split collateral: 50% to organizer, 50% to slashed_stakes for redistribution
        let organizer_share = self.member_account.collateral
            .checked_div(2)
            .ok_or(CustomError::ArithmeticUnderflow)?;
        let redistribution_share = self.member_account.collateral
            .checked_sub(organizer_share)
            .ok_or(CustomError::ArithmeticUnderflow)?;

        // Transfer organizer share
        let seeds = &[
            b"cycle",
            self.organizer.key.as_ref(),
            &[self.cycle.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: self.cycle_token_account.to_account_info(),
            to: self.organizer_token_account.to_account_info(),
            authority: self.cycle.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        anchor_spl::token::transfer(cpi_ctx, organizer_share)?;

        // Update slashed stakes
        self.cycle.slashed_stakes = self.cycle.slashed_stakes
            .checked_add(redistribution_share)
            .ok_or(CustomError::ArithmeticOverflow)?;

        // Clear member collateral
        self.member_account.collateral = 0;

        Ok(())
    }
}