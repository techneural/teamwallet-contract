use anchor_lang::prelude::*;

/// SwapIntent stores the INTENT to swap, not the route.
/// The actual swap route is computed at execution time.
#[account]
pub struct SwapIntent {
    pub team_wallet: Pubkey,
    pub proposer: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub slippage_bps: u16,
    
    pub votes_for: u8,
    pub votes_against: u8,
    pub voters_voted: Vec<u8>,
    pub snapshot_voters: Vec<Pubkey>,
    
    pub executed: bool,
    pub cancelled: bool,
    pub created_at: i64,
    pub expires_at: i64,
    pub approved: bool,
    pub approved_at: i64,
    pub execution_window_seconds: i64,
    
    pub bump: u8,
    pub nonce: Pubkey,
}

impl SwapIntent {
    pub const MAX_VOTERS: usize = 15;
    pub const MAX_CONTRIBUTORS: usize = 15;
    pub const MAX_SNAPSHOT: usize = Self::MAX_VOTERS + Self::MAX_CONTRIBUTORS;
    
    pub const DEFAULT_EXECUTION_WINDOW: i64 = 60 * 60; // 1 hour
    pub const DEFAULT_EXPIRY: i64 = 7 * 24 * 60 * 60;  // 7 days
    
    pub const SPACE: usize = 
        8 +                                // discriminator
        32 +                               // team_wallet
        32 +                               // proposer
        32 +                               // input_mint
        32 +                               // output_mint
        8 +                                // amount_in
        8 +                                // min_amount_out
        2 +                                // slippage_bps
        1 +                                // votes_for
        1 +                                // votes_against
        4 + Self::MAX_SNAPSHOT +           // voters_voted
        4 + (32 * Self::MAX_SNAPSHOT) +    // snapshot_voters
        1 +                                // executed
        1 +                                // cancelled
        8 +                                // created_at
        8 +                                // expires_at
        1 +                                // approved
        8 +                                // approved_at
        8 +                                // execution_window_seconds
        1 +                                // bump
        32;                                // nonce
        
    pub fn is_executable(&self, current_time: i64) -> bool {
        if !self.approved || self.executed || self.cancelled {
            return false;
        }
        let deadline = self.approved_at + self.execution_window_seconds;
        current_time <= deadline
    }
    
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time > self.expires_at
    }
    
    pub fn is_execution_expired(&self, current_time: i64) -> bool {
        if !self.approved {
            return false;
        }
        let deadline = self.approved_at + self.execution_window_seconds;
        current_time > deadline
    }
}
