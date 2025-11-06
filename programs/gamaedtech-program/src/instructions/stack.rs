use crate::error::ErrorCode;
use crate::state::*;
use crate::ALLOWED_MINT;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::str::FromStr;

pub fn stack(ctx: Context<Stack>, amount: u64) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;

    // Ensure the token mint matches the allowed mint
    let allowed_mint = Pubkey::from_str(ALLOWED_MINT).unwrap();
    require_keys_eq!(
        ctx.accounts.user_token_account.mint,
        allowed_mint,
        ErrorCode::InvalidTokenMint
    );

    // Transfer tokens from user to vault
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(cpi_ctx, amount)?;

    // Update stake account
    stake_account.staked_amount = stake_account.staked_amount.saturating_add(amount);
    stake_account.last_stake_time = Clock::get()?.unix_timestamp;
    stake_account.owner = ctx.accounts.user.key();

    Ok(())
}

#[derive(Accounts)]
pub struct Stack<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    // User's SPL token account (must exist)
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == Pubkey::from_str(ALLOWED_MINT).unwrap(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // Vault SPL token account (must exist)
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    // Custom stake account
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<StakeAccount>(),
        seeds = [b"stake-account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
