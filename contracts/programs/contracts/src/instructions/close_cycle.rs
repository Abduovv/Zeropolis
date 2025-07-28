use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct CloseCycle<'info> {
    #[account(mut)]
    pub organizer: Signer<'info>,

    #[account(
        mut,
        has_one = organizer,
        constraint = !cycle.is_active @ CustomError::CycleStillActive,
        close = organizer
    )]
    pub cycle: Account<'info, CycleAccount>,

    #[account(
        mut,
        seeds = [b"organizer", organizer.key().as_ref()],
        bump
    )]
    pub organizer_account: Account<'info, OrganizerAccount>,

    #[account(
        mut,
        constraint = member_account.cycle == cycle.key() @ CustomError::InvalidCycle,
        constraint = member_account.is_active @ CustomError::MemberNotActive,
        close = member
    )]
    pub member_account: Option<Account<'info, MemberAccount>>,

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

    #[account(
        mut,
        associated_token::mint = cycle.token_mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Option<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseCycle<'info> {
    pub fn close_cycle(&mut self) -> Result<()> {
        let seeds = &[
            b"cycle",
            self.organizer.key.as_ref(),
            &[self.cycle.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // Refund organizer stake
        let cpi_accounts = Transfer {
            from: self.cycle_token_account.to_account_info(),
            to: self.organizer_token_account.to_account_info(),
            authority: self.cycle.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        anchor_spl::token::transfer(cpi_ctx, self.cycle.organizer_stake)?;

        // Update organizer account
        self.organizer_account.total_cycles = self.organizer_account.total_cycles
            .checked_sub(1)
            .ok_or(CustomError::ArithmeticUnderflow)?;
        self.organizer_account.locked_stake = self.organizer_account.locked_stake
            .checked_sub(self.cycle.organizer_stake)
            .ok_or(CustomError::ArithmeticUnderflow)?;

        // Refund member collateral and distribute slashed stakes
        if let (Some(ref mut member_account), Some(ref recipient_token_account)) = (
            self.member_account.as_ref(),
            self.recipient_token_account.as_ref(),
        ) {
            require!(
                member_account.member == self.recipient.key(),
                CustomError::InvalidMember
            );

            // Refund member collateral
            let collateral_refunded = member_account.collateral;
            if collateral_refunded > 0 {
                let cpi_accounts = Transfer {
                    from: self.cycle_token_account.to_account_info(),
                    to: recipient_token_account.to_account_info(),
                    authority: self.cycle.to_account_info(),
                };
                let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
                anchor_spl::token::transfer(cpi_ctx, collateral_refunded)?;
            }

            // Distribute slashed stakes proportionally
            if self.cycle.slashed_stakes > 0 {
                let member_share = self.cycle.slashed_stakes
                    .checked_div(self.cycle.current_participants as u64)
                    .ok_or(CustomError::ArithmeticUnderflow)?;
                if member_share > 0 {
                    let cpi_accounts = Transfer {
                        from: self.cycle_token_account.to_account_info(),
                        to: recipient_token_account.to_account_info(),
                        authority: self.cycle.to_account_info(),
                    };
                    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
                    anchor_spl::token::transfer(cpi_ctx, member_share)?;
                    self.cycle.slashed_stakes = self.cycle.slashed_stakes
                        .checked_sub(member_share)
                        .ok_or(CustomError::ArithmeticUnderflow)?;
                }
            }
        }

        Ok(())
    }
}