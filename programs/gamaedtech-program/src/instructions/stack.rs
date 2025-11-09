use crate::error::ErrorCode;
use crate::state::*;
use crate::ALLOWED_MINT;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, TransferChecked};
use std::str::FromStr;

pub fn process_stack(ctx: Context<Stack>, amount: u64) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;

    // Get decimals from the mint
    let decimals = ctx.accounts.mint.decimals;

    // Ensure the token mint matches the allowed mint
    let allowed_mint = Pubkey::from_str(ALLOWED_MINT).unwrap();
    require_keys_eq!(
        ctx.accounts.user_token_account.mint,
        allowed_mint,
        ErrorCode::InvalidTokenMint
    );

    // Transfer tokens from user to vault using Token-2022 CPI
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.user_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );

    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

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

    // User's Token-2022 account
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == Pubkey::from_str(ALLOWED_MINT).unwrap(),
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // Vault Token-2022 account
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    // Mint account of the Token-2022 token
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    // Custom stake account
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<StakeAccount>(),
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    // Token-2022 program
    pub token_program: Program<'info, Token2022>,

    pub system_program: Program<'info, System>,
}
