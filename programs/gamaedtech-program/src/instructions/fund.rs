use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
// use squads_multisig_program::cpi;
// use squads_multisig_program::cpi::accounts::VaultTransactionCreate;
// use squads_multisig_program::VaultTransactionCreateArgs;

#[derive(Accounts)]
pub struct RequestFund<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>, // The proposal requesting funding

    /// CHECK: The Squads multisig account (vault/multisig)
    pub multisig: UncheckedAccount<'info>,

    /// CHECK: The transaction account to be created in Squads
    #[account(mut)]
    pub transaction: UncheckedAccount<'info>,

    /// The wallet (user) paying for rent and signing
    #[account(mut)]
    pub creator: Signer<'info>,

    /// The wallet (user) paying for rent
    #[account(mut)]
    pub rent_payer: Signer<'info>,

    /// Squads program (generic type works for CPI)
    // pub squads_program: Program<'info, squads_multisig_program::Multisig>,
    pub system_program: Program<'info, System>,
}

pub fn process_request_fund(ctx: Context<RequestFund>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;

    // Prevent duplicate funding requests
    require!(!proposal.is_fund_requested, ErrorCode::AlreadyRequested);

    // Ensure proposal has expired (voting period ended)
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp > proposal.expires_at,
        ErrorCode::VotingStillActive
    );

    // Ensure majority agrees
    require!(
        proposal.agree_votes > proposal.disagree_votes,
        ErrorCode::ProposalRejected
    );

    // ---- Perform CPI to Squads ----
    // let cpi_accounts = VaultTransactionCreate {
    // multisig: ctx.accounts.multisig.to_account_info(),
    // transaction: ctx.accounts.transaction.to_account_info(),
    // creator: ctx.accounts.creator.to_account_info(),
    // rent_payer: ctx.accounts.rent_payer.to_account_info(),
    // system_program: ctx.accounts.system_program.to_account_info().clone().into(),
    // };

    // let cpi_ctx = CpiContext::new(ctx.accounts.squads_program.to_account_info(), cpi_accounts);
    // cpi::vault_transaction_create(cpi_ctx, args)?;

    // Mark as requested
    // proposal.is_fund_requested = true;

    Ok(())
}
