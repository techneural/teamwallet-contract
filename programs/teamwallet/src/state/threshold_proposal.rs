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
    /// Snapshot of eligible voters at proposal creation time
    pub snapshot_voters: Vec<Pubkey>,
    /// Pubkeys of members who have already voted (prevents double-voting)
    pub voters_voted: Vec<Pubkey>,
    /// Number of approve votes cast
    pub votes_for: u8,
    /// Number of reject votes cast
    pub votes_against: u8,
    /// Whether the proposal was auto-cancelled due to irrecoverable rejection
    pub cancelled: bool,
}

impl ThresholdProposal {
    pub const MAX_VOTERS: usize = 15;

    pub const SPACE: usize =
        8 +                              // discriminator
        32 +                             // team_wallet
        32 +                             // proposer
        1 +                              // new_threshold
        1 +                              // old_threshold
        1 +                              // executed
        1 +                              // bump
        32 +                             // nonce
        4 + (32 * Self::MAX_VOTERS) +    // snapshot_voters Vec<Pubkey>
        4 + (32 * Self::MAX_VOTERS) +    // voters_voted Vec<Pubkey>
        1 +                              // votes_for
        1 +                              // votes_against
        1;                               // cancelled
}