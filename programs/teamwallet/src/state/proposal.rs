
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
    // Stores the INDEX of each voter in snapshot_voters — 1 byte per member
    // instead of 32 bytes per member (Vec<Pubkey>)
    pub voters_voted: Vec<u8>,
    // Full pubkeys stored once at proposal creation
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
    pub const MAX_VOTERS: usize = 15;

    pub const SPACE: usize =
        8 +                            // discriminator
        32 +                           // team_wallet
        32 +                           // proposer
        8 +                            // amount
        32 +                           // recipient
        1 +                            // is_token_transfer
        33 +                           // mint Option<Pubkey>
        1 +                            // votes_for
        1 +                            // votes_against
        4 + Self::MAX_VOTERS +         // voters_voted Vec<u8>  → 1 byte per member
        4 + (32 * Self::MAX_VOTERS) +  // snapshot_voters Vec<Pubkey> → 32 bytes per member
        1 +                            // executed
        1 +                            // bump
        1 +                            // is_swap_proposal
        33 +                           // input_mint Option<Pubkey>
        33 +                           // output_mint Option<Pubkey>
        9 +                            // min_output_amount Option<u64>
        3 +                            // slippage_bps Option<u16>
        8 +                            // nonce
        1;                             // ready_to_execute
}
