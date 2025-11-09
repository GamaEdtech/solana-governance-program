use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{self, TransferChecked};

pub fn process_unstack(ctx: Context<Unstack>, amount: u64) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;
    let now = Clock::get()?.unix_timestamp;

    // Only owner can unstack
    require_keys_eq!(
        stake_account.owner,
        ctx.accounts.user.key(),
        ErrorCode::Unauthorized
    );

    // Check enough stake
    require!(
        stake_account.staked_amount >= amount,
        ErrorCode::InsufficientStake
    );

    // Set pending unstake info
    stake_account.pending_unstake = amount;
    stake_account.unstake_requested_at = now;

    Ok(())
}

#[derive(Accounts)]
pub struct Unstack<'info> {
    #[account(
        mut,
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    pub user: Signer<'info>,
}

pub fn process_claim_unstake(ctx: Context<ClaimUnstake>) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;
    let now = Clock::get()?.unix_timestamp;
    const COOLDOWN_PERIOD: i64 = 3 * 24 * 60 * 60; // 3 days

    // Ownership check
    require_keys_eq!(
        stake_account.owner,
        ctx.accounts.user.key(),
        ErrorCode::Unauthorized
    );

    // Ensure user has a pending unstake
    require!(stake_account.pending_unstake > 0, ErrorCode::NothingToClaim);

    // Enforce 3-day cooldown
    require!(
        now.saturating_sub(stake_account.unstake_requested_at) >= COOLDOWN_PERIOD,
        ErrorCode::CooldownActive
    );

    // Transfer tokens: vault → user using Token-2022 CPI
    let seeds: &[&[u8]] = &[b"vault-authority".as_ref(), &[ctx.bumps.vault_authority]];
    let signer = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.vault_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer,
    );

    token_interface::transfer_checked(
        cpi_ctx,
        stake_account.pending_unstake,
        ctx.accounts.mint.decimals,
    )?;

    // Update stake info
    stake_account.staked_amount = stake_account
        .staked_amount
        .saturating_sub(stake_account.pending_unstake);
    stake_account.pending_unstake = 0;
    stake_account.unstake_requested_at = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimUnstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: vault authority PDA
    #[account(
        seeds = [b"vault-authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    /// Mint account of the Token-2022 token
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    pub token_program: Program<'info, Token2022>,
}
