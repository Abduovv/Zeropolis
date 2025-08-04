use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, TokenAccount, Transfer},
};
use crate::{error::CustomError, util::calculate_payout_amount, CycleAccount, MemberAccount};

#[derive(Accounts)]
pub struct TriggerPayout<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        mut,
        has_one = organizer,
        seeds = [b"cycle", organizer.key.as_ref(), cycle.nonces.to_le_bytes().as_ref()],
        bump = cycle.bump,
        constraint = cycle.is_active @ CustomError::CycleNotActive
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = cycle
    )]
    pub cycle_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = organizer
    )]
    pub organizer_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = member_account.member == recipient.key() @ CustomError::InvalidPayoutRecipient,
        constraint = member_account.is_active @ CustomError::MemberNotActive
    )]
    pub member_account: Account<'info, MemberAccount>,

    #[account(mut)]
    pub recipient: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> TriggerPayout<'info> {
    pub fn trigger_payout(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp >= self.cycle.next_round_time,
            CustomError::PayoutTooEarly
        );
        require!(
            self.cycle.current_round < self.cycle.round_count * self.cycle.contributions_per_payout,
            CustomError::CycleComplete
        );

        // Update round and next round time
        self.cycle.current_round = self.cycle.current_round
            .checked_add(1)
            .ok_or(CustomError::ArithmeticOverflow)?;
        self.cycle.next_round_time = self.cycle.next_round_time
            .checked_add(self.cycle.contribution_interval)
            .ok_or(CustomError::ArithmeticOverflow)?;

        // Check if this is a payout round
        if self.cycle.current_round % self.cycle.contributions_per_payout == 0 {
            let payout_index = (self.cycle.current_round / self.cycle.contributions_per_payout)
                .checked_sub(1)
                .ok_or(CustomError::ArithmeticUnderflow)?;
            require!(
                self.cycle.payout_order[payout_index as usize] == self.recipient.key(),
                CustomError::InvalidPayoutRecipient
            );

            // Calculate payout and fee
            let pot_amount = self.cycle.pot_amount;
            let payout_amount = calculate_payout_amount(pot_amount, self.cycle.organizer_fee_bps)?;
            let organizer_fee = pot_amount
                .checked_sub(payout_amount)
                .ok_or(CustomError::ArithmeticUnderflow)?;

            // Transfer payout to recipient
            let seeds = &[
                b"cycle",
                self.organizer.key.as_ref(),
                &self.cycle.nonces.to_le_bytes(),
                &[self.cycle.bump],
            ];
            let signer_seeds = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: self.cycle_token_account.to_account_info(),
                to: self.recipient_token_account.to_account_info(),
                authority: self.cycle.to_account_info(),
            };
            let cpi_program = self.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer_seeds);
            anchor_spl::token::transfer(cpi_ctx, payout_amount)?;

            // Transfer organizer fee
            let cpi_accounts = Transfer {
                from: self.cycle_token_account.to_account_info(),
                to: self.organizer_token_account.to_account_info(),
                authority: self.cycle.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            anchor_spl::token::transfer(cpi_ctx, organizer_fee)?;

            // Mark recipient as having received payout
            self.member_account.payout_received = true;
        }

        // Deactivate cycle if complete
        if self.cycle.current_round >= self.cycle.round_count * self.cycle.contributions_per_payout {
            self.cycle.is_active = false;
        }

        Ok(())
    }
}