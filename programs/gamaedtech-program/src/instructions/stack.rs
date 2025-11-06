use crate::state::*;
use anchor_lang::prelude::*;

pub fn stack(ctx: Context<Stack>, amount: u64) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;

    stake_account.staked_amount += amount;
    stake_account.last_stake_time = Clock::get()?.unix_timestamp;

    Ok(())
}

#[derive(Accounts)]
pub struct Stack<'info> {
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
