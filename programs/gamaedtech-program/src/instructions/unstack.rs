use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn unstack(ctx: Context<Unstack>, amount: u64) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;

    require!(
        stake_account.staked_amount >= amount,
        ErrorCode::InsufficientStake
    );

    stake_account.staked_amount -= amount;

    Ok(())
}

#[derive(Accounts)]
pub struct Unstack<'info> {
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
    pub user: Signer<'info>,
}
