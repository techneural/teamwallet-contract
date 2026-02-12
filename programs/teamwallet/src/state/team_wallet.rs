
use anchor_lang::prelude::*;

#[account]
pub struct TeamWallet {
    pub owner: Pubkey,
    pub name: String,
    pub voters: Vec<Pubkey>,
    pub contributors: Vec<Pubkey>,
    pub voter_count: u8,
    pub vote_threshold: u8,
    pub proposal_count: u64,
    pub bump: u8,
}

impl TeamWallet {
    pub const MAX_SIZE: usize =
        8 +
        32 +
        36 +
        484 +
        484 +
        1 +
        1 +
        8 +
        1;
}