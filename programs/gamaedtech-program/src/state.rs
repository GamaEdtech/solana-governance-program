use anchor_lang::prelude::*;
#[account]
pub struct Proposal {
    pub owner: Pubkey,
    pub title: String,
    pub brief: String,
    pub cate: String,
    pub reference: String,
    pub amount: u64,
    pub agree_votes: u64, // Number of "Agree" votes
    pub disagree_votes: u64,
    pub created_at: i64, // Creation timestamp in seconds
    pub expires_at: i64, // Expiration timestamp in seconds
    pub is_fund_requested: bool,
}

impl Space for Proposal {
    const INIT_SPACE: usize = 32 +        // owner
        (4 + 100) + // title
        (4 + 800) + // brief
        (4 + 50) +  // cate
        (4 + 200) + // reference
        8 +         // amount
        8 +         // agree_votes
        8 +         // disagree_votes
        8 +         // created_at
        8; // expires_at
}

#[account]
pub struct UserState {
    pub user: Pubkey,
    pub proposal_count: u64,
}

#[account]
pub struct VoteRecord {
    pub proposal_id: Pubkey, // Linked proposal ID
    pub voter: Pubkey,       // Voter's public key
    pub has_voted: bool,     // Prevent double voting
    pub vote: String,
    pub vote_power: u64,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,        // Who owns this stake
    pub staked_amount: u64,   // Total tokens staked
    pub last_stake_time: i64, // For optional cooldown or reward logic
    pub pending_unstake: u64,
    pub unstake_requested_at: i64,
}

impl Space for StakeAccount {
    const INIT_SPACE: usize = 8 + 32 + 8 + 8 + 8 + 8; // Pubkey + u64 + i64
}

#[account]
pub struct Stats {
    pub total_proposals: u64,  // total proposals created
    pub active_voters: u64,    // number of active voters
    pub proposals_passed: u64, // proposals that passed
    pub treasury_balance: u64, // SOL or token balance in treasury
    pub total_staked: u64,     // total staked amount
    pub bump: u8,              // PDA bump for this account
}

impl Space for Stats {
    const INIT_SPACE: usize = 8   // discriminator
        + 8  // total_proposals
        + 8  // active_voters
        + 8  // proposals_passed
        + 8  // treasury_balance
        + 8  // total_staked
        + 1; // bump
}
