use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct TriggerPayout<'info> {
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
        mut,
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = member_account.member == recipient.key() @ CustomError::InvalidMember,
        constraint = member_account.is_active @ CustomError::MemberNotActive
    )]
    pub member_account: Account<'info, MemberAccount>,

    #[account(mut)]
    pub recipient: AccountInfo<'info>,

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

        // Check if it's time for a payout (every contributions_per_payout rounds)
        let is_payout_round = (self.cycle.current_round + 1) % self.cycle.contributions_per_payout == 0;
        if is_payout_round {
            let payout_index = ((self.cycle.current_round + 1) / self.cycle.contributions_per_payout) - 1;
            require!(
                self.cycle.payout_order[payout_index as usize] == self.recipient.key(),
                CustomError::InvalidPayoutRecipient
            );

            // Calculate payout (total contributions over contributions_per_payout rounds)
            let total_payout = self.cycle.amount_per_user
                .checked_mul(self.cycle.current_participants as u64)
                .ok_or(CustomError::ArithmeticOverflow)?
                .checked_mul(self.cycle.contributions_per_payout as u64)
                .ok_or(CustomError::ArithmeticOverflow)?;
            let organizer_fee = total_payout
                .checked_mul(self.cycle.organizer_fee_bps as u64)
                .ok_or(CustomError::ArithmeticOverflow)?
                / 10_000;
            let recipient_payout = total_payout
                .checked_sub(organizer_fee)
                .ok_or(CustomError::ArithmeticOverflow)?;

            // Transfer payout to recipient
            let seeds = &[
                b"cycle",
                self.organizer.key.as_ref(),
                &[self.cycle.bump],
            ];
            let signer_seeds = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: self.cycle_token_account.to_account_info(),
                to: self.recipient_token_account.to_account_info(),
                authority: self.cycle.to_account_info(),
            };
            let cpi_program = self.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            anchor_spl::token::transfer(cpi_ctx, recipient_payout)?;

            // Transfer organizer fee
            let cpi_accounts = Transfer {
                from: self.cycle_token_account.to_account_info(),
                to: self.organizer_token_account.to_account_info(),
                authority: self.cycle.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
            anchor_spl::token::transfer(cpi_ctx, organizer_fee)?;

            // Update member account
            self.member_account.payout_received = true;
        }

        // Update cycle state
        self.cycle.current_round = self.cycle.current_round
            .checked_add(1)
            .ok_or(CustomError::ArithmeticOverflow)?;
        self.cycle.next_round_time = self.cycle.next_round_time
            .checked_add(self.cycle.contribution_interval)
            .ok_or(CustomError::ArithmeticOverflow)?;

        if (self.cycle.current_round / self.cycle.contributions_per_payout) >= self.cycle.round_count {
            self.cycle.is_active = false;
        }

        Ok(())
    }
}