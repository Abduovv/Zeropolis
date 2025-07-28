use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{error::CustomError, CycleAccount, MemberAccount};

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

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimCollateral<'info> {
    pub fn claim_collateral(&mut self) -> Result<()> {
        // Transfer entire collateral to slashed_stakes for redistribution
        let collateral = self.member_account.collateral;
        self.cycle.slashed_stakes = self.cycle.slashed_stakes
            .checked_add(collateral)
            .ok_or(CustomError::ArithmeticOverflow)?;

        // Clear member collateral
        self.member_account.collateral = 0;

        Ok(())
    }
}