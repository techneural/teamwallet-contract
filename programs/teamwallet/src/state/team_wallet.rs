use anchor_lang::prelude::*;

#[account]
pub struct TeamWallet {
    pub owner: Pubkey,              // 32
    pub name: String,               // 4 + max 32
    pub voters: Vec<Pubkey>,        // 4 + (32 * max 10) = 324
    pub contributors: Vec<Pubkey>,  // 4 + (32 * max 10) = 324
    pub voter_count: u8,            // 1
    pub vote_threshold: u8,         // 1 (percentage: 51 = 51%)
    pub proposal_count: u64,        // 8
    pub bump: u8,     
}


impl TeamWallet {
    pub const MAX_SIZE: usize = 
        8 +   // discriminator
        32 +  // owner
        36 +  // name (4 + 32)
        324 + // voters
        324 + // contributors
        1 +   // voter_count
        1 +   // vote_threshold
        8 +   // proposal_count
        1;    // bump
        
}