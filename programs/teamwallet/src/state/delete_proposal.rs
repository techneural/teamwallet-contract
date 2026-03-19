use anchor_lang::prelude::*;

#[account]
pub struct DeleteProposal {
    pub team_wallet: Pubkey,
    pub proposer: Pubkey,
    pub program_id: Pubkey,
    pub spill_account: Pubkey,
    pub votes_for: u8,
    pub votes_against: u8,
    pub voters_voted: Vec<Pubkey>,
    pub executed: bool,
    pub cancelled: bool,
    pub bump: u8,
    
    // Expiry fields
    pub created_at: i64,
    pub expires_at: i64,
}

impl DeleteProposal {
    pub const MAX_VOTERS: usize = 15;
    
    /// Default expiry: 7 days
    pub const DEFAULT_EXPIRY: i64 = 7 * 24 * 60 * 60;
    
    pub const MAX_SIZE: usize =
        8 +                              // discriminator
        32 +                             // team_wallet
        32 +                             // proposer
        32 +                             // program_id
        32 +                             // spill_account
        1 +                              // votes_for
        1 +                              // votes_against
        4 + (32 * Self::MAX_VOTERS) +    // voters_voted
        1 +                              // executed
        1 +                              // cancelled
        1 +                              // bump
        8 +                              // created_at
        8;                               // expires_at
        
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time > self.expires_at
    }
}
