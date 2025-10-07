use anchor_lang::prelude::*;

#[account]
pub struct TeamWallet {
    pub owner: Pubkey,
    pub name: String,
    pub vote_threshold: u8,
    pub voter_count: u8,
    pub voters: Vec<Pubkey>,
    pub contributors: Vec<Pubkey>,
    pub bump: u8,
}