use anchor_lang::prelude::*;

#[account]
pub struct Proposal {
    pub team_wallet: Pubkey,
    pub proposer: Pubkey,
    pub amount: u64,
    pub recipient: Pubkey,
    pub is_token_transfer: bool,
    pub mint: Option<Pubkey>,
    pub votes_for: u8,
    pub votes_against: u8,
    pub voters_voted: Vec<Pubkey>,
    pub executed: bool,
    pub bump: u8,
}