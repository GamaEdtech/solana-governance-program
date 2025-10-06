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
