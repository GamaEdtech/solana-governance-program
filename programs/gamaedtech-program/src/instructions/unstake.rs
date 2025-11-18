use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

// ========== UNSTAKE REQUEST ==========

pub fn process_unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;
    let now = Clock::get()?.unix_timestamp;

    // Only owner can unstake
    require_keys_eq!(
        stake_account.owner,
        ctx.accounts.user.key(),
        ErrorCode::Unauthorized
    );

    require!(
        stake_account.pending_unstake == 0,
        ErrorCode::AlreadyUnstaking
    );

    // Check enough stake
    require!(
        stake_account.staked_amount >= amount,
        ErrorCode::InsufficientStake
    );

    // Set pending unstake info
    stake_account.pending_unstake = amount;
    stake_account.unstake_requested_at = now;

    // --- Update stats ---
    let stats = &mut ctx.accounts.stats;
    stats.total_staked = stats.total_staked.saturating_sub(amount);

    Ok(())
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(
        mut,
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"stats"],
        bump
    )]
    pub stats: Account<'info, Stats>,

    pub user: Signer<'info>,
}

// ========== CLAIM AFTER COOLDOWN ==========

pub fn process_claim_unstake(ctx: Context<ClaimUnstake>) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;
    let now = Clock::get()?.unix_timestamp;
    const COOLDOWN_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days

    // Ownership check
    require_keys_eq!(
        stake_account.owner,
        ctx.accounts.user.key(),
        ErrorCode::Unauthorized
    );

    // Ensure user has a pending unstake
    require!(stake_account.pending_unstake > 0, ErrorCode::NothingToClaim);

    // Enforce 7-day cooldown
    require!(
        now.saturating_sub(stake_account.unstake_requested_at) >= COOLDOWN_PERIOD,
        ErrorCode::CooldownActive
    );

    // Prepare signer seeds
    let seeds: &[&[u8]] = &[b"vault-authority".as_ref(), &[ctx.bumps.vault_authority]];
    let signer = &[&seeds[..]];

    // Transfer tokens: vault → user
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
    let unstake_amount = stake_account.pending_unstake;

    stake_account.staked_amount = stake_account
        .staked_amount
        .saturating_sub(stake_account.pending_unstake);
    stake_account.pending_unstake = 0;
    stake_account.unstake_requested_at = 0;

    // --- Update stats ---
    let stats = &mut ctx.accounts.stats;
    stats.treasury_balance = stats.treasury_balance.saturating_sub(unstake_amount);

    // If user has no stake left, reduce active_voters
    if stake_account.staked_amount == 0 && stats.active_voters > 0 {
        stats.active_voters = stats.active_voters.saturating_sub(1);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimUnstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    // User token account (can be Token or Token-2022)
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    // Vault token account (can be Token or Token-2022)
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: vault authority PDA
    #[account(
        seeds = [b"vault-authority"],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    // Token mint (can be Token or Token-2022)
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"stats"],
        bump
    )]
    pub stats: Account<'info, Stats>,

    // Token program (TokenInterface supports both Token and Token-2022)
    pub token_program: Interface<'info, TokenInterface>,
}
