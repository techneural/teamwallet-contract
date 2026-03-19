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
    pub voters_voted: Vec<u8>,
    pub snapshot_voters: Vec<Pubkey>,
    pub executed: bool,
    pub cancelled: bool,
    pub bump: u8,
    
    // Expiry fields
    pub created_at: i64,
    pub expires_at: i64,
}

impl Proposal {
    pub const MAX_VOTERS: usize = 15;
    pub const MAX_CONTRIBUTORS: usize = 15;
    pub const MAX_SNAPSHOT: usize = Self::MAX_VOTERS + Self::MAX_CONTRIBUTORS;
    
    /// Default expiry: 7 days
    pub const DEFAULT_EXPIRY: i64 = 7 * 24 * 60 * 60;

    pub const SPACE: usize =
        8 +                                // discriminator
        32 +                               // team_wallet
        32 +                               // proposer
        8 +                                // amount
        32 +                               // recipient
        1 +                                // is_token_transfer
        33 +                               // mint Option<Pubkey>
        1 +                                // votes_for
        1 +                                // votes_against
        4 + Self::MAX_SNAPSHOT +           // voters_voted Vec<u8>
        4 + (32 * Self::MAX_SNAPSHOT) +    // snapshot_voters Vec<Pubkey>
        1 +                                // executed
        1 +                                // cancelled
        1 +                                // bump
        8 +                                // created_at
        8;                                 // expires_at
        
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time > self.expires_at
    }
}
