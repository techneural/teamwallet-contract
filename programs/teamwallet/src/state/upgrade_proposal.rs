use anchor_lang::prelude::*;

#[account]
pub struct UpgradeProposal {
    pub team_wallet: Pubkey,
    pub proposer: Pubkey,
    pub new_buffer: Pubkey,
    pub votes_for: u8,
    pub votes_against: u8,
    pub voters_voted: Vec<Pubkey>,
    pub executed: bool,
    pub bump: u8,
}