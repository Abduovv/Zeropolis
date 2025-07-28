use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

use crate::{error::CustomError, CycleAccount, MemberAccount};

#[derive(Accounts)]
pub struct ExitCycle<'info> {
    #[account(mut)]
    pub member: Signer<'info>,

    #[account(
        mut,
        has_one = organizer
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        mut,
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = member_account.member == member.key() @ CustomError::InvalidMember,
        constraint = member_account.is_active @ CustomError::MemberNotActive,
        close = member
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

    /// CHECK
    #[account(mut)]
    pub organizer: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ExitCycle<'info> {
    pub fn exit_cycle(&mut self) -> Result<()> {
        require!(
            self.cycle.current_round == 0,
            CustomError::CycleAlreadyStarted
        );

        // Refund collateral
        let seeds = &[
            b"cycle",
            self.organizer.key.as_ref(),
            &[self.cycle.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: self.cycle_token_account.to_account_info(),
            to: self.member_token_account.to_account_info(),
            authority: self.cycle.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        anchor_spl::token::transfer(cpi_ctx, self.member_account.collateral)?;

        // Update cycle
        self.cycle.current_participants = self.cycle.current_participants
            .checked_sub(1)
            .ok_or(CustomError::ArithmeticUnderflow)?;

        // Remove member from payout order
        self.cycle.payout_order.retain(|&x| x != self.member.key());
        self.member_account.is_active = false;

        Ok(())
    }
}