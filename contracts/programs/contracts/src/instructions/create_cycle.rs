use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};
use crate::{constants::USDT_MINT_ADRESS, error::CustomError, util::{calculate_organizer_stake, calculate_payout_amount, calculate_pot_amount}, CycleAccount, OrganizerAccount};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateCycleArgs {
    pub amount_per_user: u64,
    pub max_participants: u8,
    pub contribution_interval: i64,
    pub contributions_per_payout: u8,
    pub round_count: u8,
}

#[derive(Accounts)]
#[instruction(args: CreateCycleArgs, nonces: u8)]
pub struct CreateCycle<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        init,
        payer = organizer,
        space = 8 + CycleAccount::INIT_SPACE,
        seeds = [b"cycle", organizer.key().as_ref(), nonces.to_le_bytes().as_ref()],
        bump
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        init_if_needed,
        payer = organizer,
        seeds = [b"organizer", organizer.key().as_ref()],
        bump,
        space = 8 + OrganizerAccount::INIT_SPACE,
        constraint = organizer_account.total_cycles < 5 @ CustomError::TooManyCycles
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
        constraint = token_mint.key() == USDT_MINT_ADRESS @ CustomError::InvalidTokenMint
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
        nonces: u8
    ) -> Result<()> {
        require!(args.amount_per_user > 0, CustomError::InvalidAmountPerUser);
        require!(args.contribution_interval > 0, CustomError::InvalidContributionInterval);
        require!(args.round_count >= 1, CustomError::InvalidRoundCount);
        let clock = Clock::get()?;

        // Enforce member limits: 2 <= max_participants <= 10
        require!(
            args.max_participants >= 2 && args.max_participants <= 10,
            CustomError::InvalidMemberCount
        );

        // Calculate pot and stakes using util functions
        let pot_amount = calculate_pot_amount(
            args.amount_per_user,
            args.max_participants,
            args.contributions_per_payout,
        )?;
        let organizer_fee_bps = 100; // Fixed at 1%
        let payout_amount = calculate_payout_amount(pot_amount, organizer_fee_bps)?;
        let required_organizer_stake = calculate_organizer_stake(pot_amount)?;
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

        // Initialize cycle account with empty payout_order
        self.cycle.set_inner(CycleAccount {
            organizer: self.organizer.key(),
            token_mint: self.token_mint.key(),
            amount_per_user: args.amount_per_user,
            max_participants: args.max_participants,
            organizer_fee_bps,
            contribution_interval: args.contribution_interval,
            contributions_per_payout: args.contributions_per_payout,
            round_count: args.round_count,
            payout_order: Vec::new(), // Empty, filled during join_cycle
            created_at,
            nonces,
            bump: bumps.cycle,
            current_participants: 0,
            is_active: false,
            current_round: 0,
            next_round_time: created_at + args.contribution_interval,
            organizer_stake: required_organizer_stake,
            pot_amount,
            payout_amount,
            slashed_stakes: 0,
        });

        Ok(())
    }
}

