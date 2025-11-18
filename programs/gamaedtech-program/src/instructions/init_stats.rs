use crate::error::ErrorCode;
use crate::state::Stats;
use crate::ADMIN;
use anchor_lang::prelude::*;
use std::str::FromStr;

#[derive(Accounts)]
pub struct InitializeStats<'info> {
    #[account(
        init,                               // creates the account
        payer = authority,                  // authority pays for rent
        seeds = [b"stats"],                 // PDA seed
        bump,                               // PDA bump
        space = Stats::INIT_SPACE,
    )]
    pub stats: Account<'info, Stats>,

    #[account(mut)]
    pub authority: Signer<'info>, // must be admin

    pub system_program: Program<'info, System>,
}

pub fn process_init_stats(ctx: Context<InitializeStats>) -> Result<()> {
    let stats = &mut ctx.accounts.stats;

    // Only admin can initialize
    let admin_pubkey = Pubkey::from_str(ADMIN).map_err(|_| ErrorCode::Unauthorized)?;
    require!(
        ctx.accounts.authority.key() == admin_pubkey,
        ErrorCode::Unauthorized
    );

    // Initialize fields
    stats.total_proposals = 8;
    stats.active_voters = 0;
    stats.proposals_passed = 5;
    stats.treasury_balance = 0;
    stats.total_staked = 0;
    stats.total_rewards = 0;
    stats.total_claimed_rewards = 0;

    // Store bump
    stats.bump = ctx.bumps.stats;

    Ok(())
}

#[derive(Accounts)]
pub struct ReallocateStats<'info> {
    #[account(
        mut,
        seeds = [b"stats"],
        bump,
        realloc = Stats::INIT_SPACE,
        realloc::payer = authority,
        realloc::zero = false
    )]
    pub stats: Account<'info, Stats>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process_realloc_stats(ctx: Context<ReallocateStats>) -> Result<()> {
    // Only admin
    let admin_pubkey = Pubkey::from_str(crate::ADMIN).map_err(|_| ErrorCode::Unauthorized)?;
    require!(
        ctx.accounts.authority.key() == admin_pubkey,
        ErrorCode::Unauthorized
    );

    Ok(())
}

#[derive(Accounts)]
pub struct CloseStats<'info> {
    #[account(
        mut,
        seeds = [b"stats"],
        bump = stats.bump,
        close = authority        // sends lamports to authority
    )]
    pub stats: Account<'info, Stats>,

    #[account(mut)]
    pub authority: Signer<'info>, // must be admin

    pub system_program: Program<'info, System>,
}

pub fn process_close_stats(ctx: Context<CloseStats>) -> Result<()> {
    let admin_pubkey = Pubkey::from_str(ADMIN).map_err(|_| ErrorCode::Unauthorized)?;
    require!(
        ctx.accounts.authority.key() == admin_pubkey,
        ErrorCode::Unauthorized
    );

    // Nothing else needed. Anchor handles closing.
    msg!("Stats PDA closed successfully.");
    Ok(())
}
