use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, TokenAccount, Transfer},
};

use crate::{error::CustomError, CycleAccount, MemberAccount};

#[derive(Accounts)]
pub struct JoinCycle<'info> {
    #[account(mut)]
    pub member: Signer<'info>,

    #[account(
        mut,    
        has_one = organizer,
        constraint = cycle.current_participants < cycle.max_participants @ CustomError::CycleFull
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        init,
        payer = member,
        space = 8 + MemberAccount::INIT_SPACE,
        seeds = [b"member", cycle.key().as_ref(), member.key().as_ref()],
        bump
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

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> JoinCycle<'info> {
    pub fn join_cycle(&mut self, bumps: JoinCycleBumps) -> Result<()> {
        // Calculate required member stake (100% of payout)
        let required_member_stake = self.cycle.payout_amount;
        require!(
            self.member_token_account.amount >= required_member_stake,
            CustomError::InsufficientStake
        );

        // Append member to payout_order
        self.cycle.payout_order.push(self.member.key());

        // Update participant count
        self.cycle.current_participants = self.cycle.current_participants
            .checked_add(1)
            .ok_or(CustomError::ArithmeticOverflow)?;

        // Activate cycle if full
        if self.cycle.current_participants == self.cycle.max_participants {
            self.cycle.is_active = true;
        }

        // Initialize member account
        self.member_account.set_inner(MemberAccount {
            cycle: self.cycle.key(),
            member: self.member.key(),
            contributions_made: 0,
            payout_received: false,
            collateral: required_member_stake,
            is_active: true,
            bump: bumps.member_account,
        });

        // Transfer collateral to cycle token account
        let cpi_accounts = Transfer {
            from: self.member_token_account.to_account_info(),
            to: self.cycle_token_account.to_account_info(),
            authority: self.member.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::transfer(cpi_ctx, required_member_stake)?;

        Ok(())
    }
}

