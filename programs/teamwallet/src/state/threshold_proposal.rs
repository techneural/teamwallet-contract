use anchor_lang::prelude::*;

#[account]
pub struct ThresholdProposal {
    /// The team wallet this proposal belongs to
    pub team_wallet: Pubkey,
    /// Who created this proposal (must be owner)
    pub proposer: Pubkey,
    /// The new threshold value being proposed
    pub new_threshold: u8,
    /// The threshold value at time of proposal creation
    pub old_threshold: u8,
    /// Whether the proposal has been executed on-chain
    pub executed: bool,
    /// PDA bump
    pub bump: u8,
    /// Random nonce pubkey used as PDA seed (stored so client can verify)
    pub nonce: Pubkey,
}

impl ThresholdProposal {
    pub const SPACE: usize =
        8 +   // discriminator
        32 +  // team_wallet
        32 +  // proposer
        1 +   // new_threshold
        1 +   // old_threshold
        1 +   // executed
        1 +   // bump
        32;   // nonce
}