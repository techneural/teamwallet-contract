// FILE: teamwallet/src/state/team_wallet.rs
// CHANGE: Added `lookup_table: Pubkey` field + updated MAX_SIZE (+32 bytes)

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
    /// Address Lookup Table address created at initialization.
    /// Stored here so the frontend can always find it from chain.
    pub lookup_table: Pubkey,
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
        8 +                              // proposal_count
        1 +                              // bump
        32;                              // lookup_table (NEW)
}