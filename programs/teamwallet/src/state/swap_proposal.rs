use anchor_lang::prelude::*;

#[account]
pub struct SwapProposal {
    pub team_wallet: Pubkey,           // Team wallet that owns this proposal
    pub proposal_id: Pubkey,           // Unique proposal ID
    pub proposer: Pubkey,              // Who created the proposal
    pub amount_in: u64,                // Amount to swap
    pub min_amount_out: u64,           // Minimum amount to receive (slippage protection)
    pub input_mint: Pubkey,            // Token to swap from
    pub output_mint: Pubkey,           // Token to swap to
    pub votes_for: u8,                 // Number of votes in favor
    pub votes_against: u8,             // Number of votes against
    pub voters: Vec<Pubkey>,           // Who has voted
    pub executed: bool,                // Has this been executed?
    pub created_at: i64,               // Timestamp
    pub bump: u8,                      // PDA bump
}

impl SwapProposal {
    pub const LEN: usize = 8 +         // discriminator
        32 +                            // team_wallet
        32 +                            // proposal_id
        32 +                            // proposer
        8 +                             // amount_in
        8 +                             // min_amount_out
        32 +                            // input_mint
        32 +                            // output_mint
        1 +                             // votes_for
        1 +                             // votes_against
        4 + (32 * 10) +                // voters (vec with max 10)
        1 +                             // executed
        8 +                             // created_at
        1;                              // bump
}