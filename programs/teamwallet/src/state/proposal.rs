
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
    pub snapshot_voters: Vec<Pubkey>,
    pub executed: bool,
    pub bump: u8,
    pub is_swap_proposal: bool,
    pub input_mint: Option<Pubkey>,
    pub output_mint: Option<Pubkey>,
    pub min_output_amount: Option<u64>,
    pub slippage_bps: Option<u16>,
    pub nonce: u64,
    pub ready_to_execute: bool,
}

impl Proposal {
    pub const MAX_VOTERS: usize = 20;

    pub const SPACE: usize =
        8 +
        32 +
        32 +
        8 +
        32 +
        1 +
        33 +
        1 +
        1 +
        4 + (32 * Self::MAX_VOTERS) +
        4 + (32 * Self::MAX_VOTERS) +
        1 +
        1 +
        1 +
        33 +
        33 +
        9 +
        3 +
        8 +
        1;
}