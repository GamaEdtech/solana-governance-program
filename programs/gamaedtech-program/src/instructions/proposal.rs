use crate::error::ErrorCode;
use crate::state::{Proposal, StakeAccount, Stats, UserState};
use crate::ADMIN;
use anchor_lang::prelude::*;
use std::str::FromStr;

//Create proposal
#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    // The user state account, created if it doesn't exist yet
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8, // discriminator + Pubkey + u64
        seeds = [b"user_state", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    // The proposal account as a PDA, unique per user using proposal_count
    #[account(
        init,
        payer = user,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [
            b"proposal",
            user.key().as_ref(),
            user_state.proposal_count.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
        constraint = stake_account.staked_amount >= 50_000_000_000_000 @ ErrorCode::InsufficientStake
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"stats"],
        bump
    )]
    pub stats: Account<'info, Stats>,

    pub system_program: Program<'info, System>,
}

pub fn proccess_create_proposal(
    ctx: Context<SubmitProposal>,
    title: String,
    brief: String,
    cate: String,
    reference: String,
    amount: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let user_state = &mut ctx.accounts.user_state;
    let stats = &mut ctx.accounts.stats;
    let user = &ctx.accounts.user;
    let stake_info = &mut ctx.accounts.stake_account;

    proposal.owner = user.key();
    proposal.title = title;
    proposal.brief = brief;
    proposal.cate = cate;
    proposal.reference = reference;
    proposal.amount = amount;

    proposal.created_at = Clock::get()?.unix_timestamp;
    proposal.expires_at = proposal.created_at + 3600 * 24 * 7;

    // Increment user's proposal count so next proposal PDA is unique
    user_state.proposal_count = user_state
        .proposal_count
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    // Reward proposal with +1% of their stake
    let reward = stake_info.staked_amount / 100; // 1%
    stake_info.pending_rewards = stake_info.pending_rewards.saturating_add(reward);

    // --- Update stats ---
    stats.total_proposals = stats
        .total_proposals
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    stats.total_rewards = stats.total_rewards.saturating_add(reward);

    Ok(())
}

//End create propsoal

//Delete proposal
#[derive(Accounts)]
pub struct DeleteProposal<'info> {
    #[account(
        mut,
        close = user
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"stake_account", proposal.owner.as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"stats"],
        bump
    )]
    pub stats: Account<'info, Stats>,

    #[account(mut)]
    pub user: Signer<'info>,
}

pub fn proccess_delete_proposal(ctx: Context<DeleteProposal>) -> Result<()> {
    let proposal = &ctx.accounts.proposal;
    let user = &ctx.accounts.user;
    let stats = &mut ctx.accounts.stats;
    let stake_info = &mut ctx.accounts.stake_account;

    let admin_pubkey = Pubkey::from_str(ADMIN).unwrap();

    if proposal.owner != user.key() && user.key() != admin_pubkey {
        return Err(ErrorCode::Unauthorized.into());
    }

    // Reward proposal with -1% of their stake
    let reward = stake_info.staked_amount / 100; // 1%
    stake_info.pending_rewards = stake_info.pending_rewards.saturating_sub(reward);

    // --- Update stats ---
    stats.total_proposals = stats.total_proposals.checked_sub(1).unwrap();
    stats.total_rewards = stats.total_rewards.saturating_sub(reward);

    Ok(())
}

//End Delete propsoal
