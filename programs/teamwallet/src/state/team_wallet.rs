
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
    // Max 15 voters (owner + 14 added), max 15 contributors
    pub const MAX_VOTERS: usize = 15;

    pub const MAX_SIZE: usize =
        8 +                              // discriminator
        32 +                             // owner
        36 +                             // name (4 len prefix + 32 max chars)
        4 + (32 * Self::MAX_VOTERS) +    // voters Vec<Pubkey>
        4 + (32 * Self::MAX_VOTERS) +    // contributors Vec<Pubkey>
        1 +                              // voter_count
        1 +                              // vote_threshold
        8 +                              // proposal_count (was missing!)
        1;                               // bump
}
