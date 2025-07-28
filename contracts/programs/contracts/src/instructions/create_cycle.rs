use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

use crate::{organizer_account, state::{CycleAccount, MemberAccount, OrganizerAccount}};
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateCycleArgs {
    pub amount_per_user: u64,
    pub max_participants: u8,
    pub organizer_fee_bps: u16,
    pub contribution_interval: i64,
    pub contributions_per_payout: u8,
    pub round_count: u8,
    pub payout_order: Vec<Pubkey>,
    pub token_mint: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: CreateCycleArgs)]
pub struct CreateCycle<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        init,
        payer = organizer,
        space = 8 + CycleAccount::INIT_SPACE,
        seeds = [b"cycle", organizer.key().as_ref()],
        bump
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        init_if_needed,
        payer = organizer,
        seeds = [b"organizer", organizer.key().as_ref()],
        bump,
        space = 8 + OrganizerAccount::INIT_SPACE,
    )]
    pub organizer_account: Account<'info, OrganizerAccount>,

    #[account(
        init,
        payer = organizer,
        associated_token::mint = token_mint,
        associated_token::authority = cycle
    )]
    pub cycle_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = organizer
    )]
    pub organizer_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = token_mint.key() == args.token_mint @ CustomError::InvalidTokenMint
    )]
    pub token_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateCycle<'info> {
    pub fn create_cycle(
        &mut self,
        args: CreateCycleArgs,
        bumps: CreateCycleBumps,
    ) -> Result<()> {
        let clock = Clock::get()?;

        require!(
            args.payout_order.len() as u8 == args.max_participants,
            CustomError::InvalidPayoutOrder
        );
        self.organizer_account.load_mut()?;
        let total_cycles = organizer_account_data.total_cycles;
        require!(
            total_cycles < 5,
            CustomError::TooManyCycles
        );

        // Calculate pot amount and organizer stake
        let pot_amount = args.amount_per_user
            .checked_mul(args.max_participants as u64)
            .ok_or(CustomError::ArithmeticOverflow)?
            .checked_mul(args.contributions_per_payout as u64)
            .ok_or(CustomError::ArithmeticOverflow)?;
        let required_organizer_stake = pot_amount
            .checked_mul(20)
            .ok_or(CustomError::ArithmeticOverflow)?
            / 100; // 20% of pot
        require!(
            self.organizer_token_account.amount >= required_organizer_stake,
            CustomError::InsufficientStake
        );

        let created_at = clock.unix_timestamp;

        // Update organizer account
        self.organizer_account.total_cycles = self.organizer_account.total_cycles
            .checked_add(1)
            .ok_or(CustomError::ArithmeticOverflow)?;
        self.organizer_account.locked_stake = self.organizer_account.locked_stake
            .checked_add(required_organizer_stake)
            .ok_or(CustomError::ArithmeticOverflow)?;
        self.organizer_account.last_cycle_time = created_at;

        // Transfer organizer stake to cycle token account
        let cpi_accounts = Transfer {
            from: self.organizer_token_account.to_account_info(),
            to: self.cycle_token_account.to_account_info(),
            authority: self.organizer.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::transfer(cpi_ctx, required_organizer_stake)?;

        // Initialize cycle account
        self.cycle.set_inner(CycleAccount {
            organizer: self.organizer.key(),
            token_mint: args.token_mint,
            amount_per_user: args.amount_per_user,
            max_participants: args.max_participants,
            organizer_fee_bps: args.organizer_fee_bps,
            contribution_interval: args.contribution_interval,
            contributions_per_payout: args.contributions_per_payout,
            round_count: args.round_count,
            payout_order: args.payout_order.clone(),
            created_at,
            bump: bumps.cycle,
            current_participants: 0,
            is_active: false,
            current_round: 0,
            next_round_time: created_at + args.contribution_interval,
            organizer_stake: required_organizer_stake,
            pot_amount,
            slashed_stakes: 0,
        });

        Ok(())
    }
}

