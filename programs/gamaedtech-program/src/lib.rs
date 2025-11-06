use anchor_lang::prelude::*;
declare_id!("9F77hJsRRXs7vF9UDncZKth2r5wEPgcRkEfyoZDNQ3eK");
const ALLOWED_MINT: &str = "GeutGuhcTYRf4rkbZmWDMEgjt5jHyJN4nHko38GJjQhv";

use instructions::*;
mod error;
mod instructions;
mod state;

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "GamaEdtech Governance",
    project_url: "https://gamatrain.com",
    contacts: "security@gamatrain.com",
    policy: "https://gamatrain.com/security/policy",
    preferred_languages: "en",
    source_code: "https://github.com/GamaEdtech/solana-governance-program"
}

#[program]
pub mod gamaedtech_program {

    use super::*;

    pub fn create_proposal(
        ctx: Context<SubmitProposal>,
        title: String,
        brief: String,
        cate: String,
        reference: String,
        amount: u64,
    ) -> Result<()> {
        proccess_create_proposal(ctx, title, brief, cate, reference, amount)
    }

    pub fn delete_proposal(ctx: Context<DeleteProposal>) -> Result<()> {
        proccess_delete_proposal(ctx)
    }

    pub fn vote(ctx: Context<Vote>, agree: bool) -> Result<()> {
        proccess_vote(ctx, agree)
    }
    pub fn request_fund(ctx: Context<RequestFund>) -> Result<()> {
        process_request_fund(ctx)
    }

    pub fn stack(ctx: Context<Stack>, amount: u64) -> Result<()> {
        instructions::stack::stack(ctx, amount)
    }

    pub fn unstack(ctx: Context<Unstack>, amount: u64) -> Result<()> {
        instructions::unstack::unstack(ctx, amount)
    }
}
