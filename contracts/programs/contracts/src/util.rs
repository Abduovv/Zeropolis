use anchor_lang::prelude::*;

use crate::error::CustomError;

pub fn calculate_pot_amount(
    amount_per_user: u64,
    max_participants: u8,
    contributions_per_payout: u8,
) -> Result<u64> {
    amount_per_user
        .checked_mul(max_participants as u64)
        .ok_or(CustomError::ArithmeticOverflow)?
        .checked_mul(contributions_per_payout as u64)
        .ok_or(CustomError::ArithmeticOverflow.into())
}

pub fn calculate_payout_amount(pot_amount: u64, organizer_fee_bps: u16) -> Result<u64> {
    let fee = pot_amount
        .checked_mul(organizer_fee_bps as u64)
        .ok_or(CustomError::ArithmeticOverflow)?
        / 10_000;
    pot_amount
        .checked_sub(fee)
        .ok_or(CustomError::ArithmeticUnderflow.into())
}

pub fn calculate_organizer_stake(pot_amount: u64) -> Result<u64> {
    Ok(pot_amount
        .checked_mul(20)
        .ok_or(CustomError::ArithmeticOverflow)?
        / 100)
}

pub fn calculate_member_penalty(
    collateral: u64,
    missed_rounds: u64,
    payout_received: bool,
) -> Result<u64> {
    if payout_received || missed_rounds >= 3 {
        Ok(collateral)
    } else {
        Ok(collateral
            .checked_mul(20 * missed_rounds)
            .ok_or(CustomError::ArithmeticOverflow)?
            / 100)
    }
}

pub fn calculate_organizer_penalty(
    organizer_stake: u64,
    seconds_delayed: i64,
) -> Result<u64> {
    let days_delayed = seconds_delayed / 86_400;
    let penalty_percentage = days_delayed * 10; // 10% per day
    if penalty_percentage >= 100 {
        Ok(organizer_stake)
    } else {
        Ok(organizer_stake
            .checked_mul(penalty_percentage as u64)
            .ok_or(CustomError::ArithmeticOverflow)?
            / 100)
    }
}