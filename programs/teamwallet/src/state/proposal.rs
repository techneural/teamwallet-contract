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
    pub is_swap_proposal: bool,
    pub input_mint: Option<Pubkey>,
    pub output_mint: Option<Pubkey>,
    pub min_output_amount: Option<u64>,
    pub slippage_bps: Option<u16>,
    pub nonce :u64,
    pub ready_to_execute: bool,
}
 
impl Proposal {
    pub const MAX_VOTERS: usize = 20;

    pub const SPACE: usize =
        8 +                         // discriminator
        32 +                        // team_wallet
        32 +                        // proposer
        8 +                         // amount
        32 +                        // recipient
        1 +                         // is_token_transfer
        (1 + 32) +                  // mint Option<Pubkey>
        1 +                         // votes_for
        1 +                         // votes_against
        4 + (32 * Self::MAX_VOTERS) + // voters_voted Vec<Pubkey>
        1 +                         // executed
        1 +                         // bump
        1 +                         // is_swap_proposal
        (1 + 32) +                  // input_mint
        (1 + 32) +                  // output_mint
        (1 + 8) +                   // min_output_amount
        (1 + 2) +                   // slippage_bps
        8;                          // nonce
}

 
 
 
 