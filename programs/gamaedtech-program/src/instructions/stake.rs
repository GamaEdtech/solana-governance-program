use crate::error::ErrorCode;
use crate::state::*;
use crate::ALLOWED_MINT;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use std::str::FromStr;

pub fn process_stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;
    let stats = &mut ctx.accounts.stats;

    // Get decimals from the mint
    let decimals = ctx.accounts.mint.decimals;

    // Ensure the token mint matches the allowed mint
    let allowed_mint = Pubkey::from_str(ALLOWED_MINT).unwrap();
    require_keys_eq!(
        ctx.accounts.user_token_account.mint,
        allowed_mint,
        ErrorCode::InvalidTokenMint
    );

    // Create CPI context for token transfer (Token-2022 compatible)
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.user_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );

    // Perform token transfer
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    // --- Update staking state ---
    let first_time_stake = stake_account.staked_amount == 0;

    // Update staking state
    stake_account.staked_amount = stake_account.staked_amount.saturating_add(amount);
    stake_account.last_stake_time = Clock::get()?.unix_timestamp;
    stake_account.owner = ctx.accounts.user.key();

    // --- Update stats ---
    stats.total_staked = stats.total_staked.saturating_add(amount);
    stats.treasury_balance = stats.treasury_balance.saturating_add(amount);

    if first_time_stake {
        stats.active_voters = stats.active_voters.saturating_add(1);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    /// The user performing the stake
    #[account(mut)]
    pub user: Signer<'info>,

    /// User's Token-2022 or legacy SPL token account
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == Pubkey::from_str(ALLOWED_MINT).unwrap(),
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Vault Token-2022 or legacy SPL token account (where tokens are staked)
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Token mint (Token-2022 or legacy SPL)
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Custom stake account to track user's stake
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<StakeAccount>(),
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

    /// The token program (can be Token-2022 or legacy SPL)
    pub token_program: Interface<'info, TokenInterface>,

    /// System program (for paying rent, etc.)
    pub system_program: Program<'info, System>,
}
