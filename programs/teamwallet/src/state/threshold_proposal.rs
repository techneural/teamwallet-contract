use anchor_lang::prelude::*;

#[account]
pub struct ThresholdProposal {
    pub team_wallet: Pubkey,
    pub proposer: Pubkey,
    pub new_threshold: u8,
    pub old_threshold: u8,
    pub executed: bool,
    pub cancelled: bool,
    pub bump: u8,
    pub nonce: Pubkey,
    pub snapshot_voters: Vec<Pubkey>,
    pub voters_voted: Vec<Pubkey>,
    pub votes_for: u8,
    pub votes_against: u8,
    
    // Expiry fields
    pub created_at: i64,
    pub expires_at: i64,
}

impl ThresholdProposal {
    pub const MAX_VOTERS: usize = 15;
    
    /// Default expiry: 7 days
    pub const DEFAULT_EXPIRY: i64 = 7 * 24 * 60 * 60;

    pub const SPACE: usize =
        8 +                              // discriminator
        32 +                             // team_wallet
        32 +                             // proposer
        1 +                              // new_threshold
        1 +                              // old_threshold
        1 +                              // executed
        1 +                              // cancelled
        1 +                              // bump
        32 +                             // nonce
        4 + (32 * Self::MAX_VOTERS) +    // snapshot_voters Vec<Pubkey>
        4 + (32 * Self::MAX_VOTERS) +    // voters_voted Vec<Pubkey>
        1 +                              // votes_for
        1 +                              // votes_against
        8 +                              // created_at
        8;                               // expires_at
        
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time > self.expires_at
    }
}
