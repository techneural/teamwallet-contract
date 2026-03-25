use anchor_lang::prelude::*;

/// All possible proposal actions in a single enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalAction {
    // ═══════════════════════════════════════════════════════════════════════
    // TRANSFERS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Transfer SOL from team wallet
    TransferSol {
        amount: u64,
        recipient: Pubkey,
    },
    
    /// Transfer SPL tokens from team wallet
    TransferToken {
        amount: u64,
        recipient: Pubkey,
        mint: Pubkey,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // SWAP (Jupiter/Raydium)
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Swap tokens via DEX aggregator
    Swap {
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount_in: u64,
        min_amount_out: u64,
        slippage_bps: u16,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // GOVERNANCE
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Change the vote threshold
    ChangeThreshold {
        new_threshold: u8,
    },
    
    /// Add a new voter
    AddVoter {
        voter: Pubkey,
    },
    
    /// Remove a voter
    RemoveVoter {
        voter: Pubkey,
    },
    
    /// Add a contributor
    AddContributor {
        contributor: Pubkey,
    },
    
    /// Remove a contributor
    RemoveContributor {
        contributor: Pubkey,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // PROGRAM MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Upgrade a program
    UpgradeProgram {
        program_id: Pubkey,
        buffer: Pubkey,
        spill: Pubkey,
    },
    
    /// Delete/close a program
    DeleteProgram {
        program_id: Pubkey,
        spill: Pubkey,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // TOKEN MANAGEMENT (Mint authority actions)
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Mint new tokens
    TokenMint {
        mint: Pubkey,
        amount: u64,
        recipient: Pubkey,
    },
    
    /// Burn tokens
    TokenBurn {
        mint: Pubkey,
        amount: u64,
    },
    
    /// Freeze a token account
    TokenFreeze {
        mint: Pubkey,
        account: Pubkey,
    },
    
    /// Thaw a frozen token account
    TokenThaw {
        mint: Pubkey,
        account: Pubkey,
    },
    
    /// Transfer mint authority
    TokenSetMintAuthority {
        mint: Pubkey,
        new_authority: Option<Pubkey>,
    },
    
    /// Transfer freeze authority
    TokenSetFreezeAuthority {
        mint: Pubkey,
        new_authority: Option<Pubkey>,
    },
    
    /// Update token metadata
    TokenUpdateMetadata {
        mint: Pubkey,
        name: String,
        symbol: String,
        uri: String,
    },
}

impl ProposalAction {
    /// Returns the maximum serialized size of any action variant
    pub const MAX_SIZE: usize = 1 +    // enum discriminator
        32 +                            // pubkey 1
        32 +                            // pubkey 2
        32 +                            // pubkey 3
        8 +                             // u64
        8 +                             // u64
        2 +                             // u16
        4 + 32 +                        // String name (max 32)
        4 + 10 +                        // String symbol (max 10)
        4 + 200;                        // String uri (max 200)
    
    /// Check if this action requires an execution window (like swaps)
    pub fn requires_execution_window(&self) -> bool {
        matches!(self, ProposalAction::Swap { .. })
    }
}

/// Unified proposal account for all action types
#[account]
pub struct Proposal {
    /// Team wallet this proposal belongs to
    pub team_wallet: Pubkey,
    
    /// Who created this proposal
    pub proposer: Pubkey,
    
    /// The action to be executed
    pub action: ProposalAction,
    
    // ═══════════════════════════════════════════════════════════════════════
    // VOTING
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Number of approve votes
    pub votes_for: u8,
    
    /// Number of reject votes
    pub votes_against: u8,
    
    /// Indices of voters who have voted (1 byte each)
    pub voters_voted: Vec<u8>,
    
    /// Snapshot of eligible voters at creation time
    pub snapshot_voters: Vec<Pubkey>,
    
    /// Snapshot of threshold at creation time (for auto-cancel logic)
    pub snapshot_threshold: u8,
    
    // ═══════════════════════════════════════════════════════════════════════
    // STATUS
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Whether proposal has been executed
    pub executed: bool,
    
    /// Whether proposal has been cancelled
    pub cancelled: bool,
    
    // ═══════════════════════════════════════════════════════════════════════
    // TIMING
    // ═══════════════════════════════════════════════════════════════════════
    
    /// When proposal was created
    pub created_at: i64,
    
    /// When proposal expires
    pub expires_at: i64,
    
    /// Whether threshold has been reached (for swap execution window)
    pub approved: bool,
    
    /// When approval threshold was reached
    pub approved_at: i64,
    
    /// Execution window in seconds (for swaps)
    pub execution_window: i64,
    
    // ═══════════════════════════════════════════════════════════════════════
    // PDA
    // ═══════════════════════════════════════════════════════════════════════
    
    pub bump: u8,
    pub nonce: Pubkey,
}

impl Proposal {
    pub const MAX_VOTERS: usize = 15;
    pub const MAX_CONTRIBUTORS: usize = 15;
    pub const MAX_SNAPSHOT: usize = Self::MAX_VOTERS + Self::MAX_CONTRIBUTORS;
    
    /// Default proposal expiry: 7 days
    pub const DEFAULT_EXPIRY: i64 = 7 * 24 * 60 * 60;
    
    /// Default execution window for swaps: 1 hour
    pub const DEFAULT_EXECUTION_WINDOW: i64 = 60 * 60;
    
    pub const SPACE: usize =
        8 +                                    // discriminator
        32 +                                   // team_wallet
        32 +                                   // proposer
        ProposalAction::MAX_SIZE +             // action
        1 +                                    // votes_for
        1 +                                    // votes_against
        4 + Self::MAX_SNAPSHOT +               // voters_voted Vec<u8>
        4 + (32 * Self::MAX_SNAPSHOT) +        // snapshot_voters Vec<Pubkey>
        1 +                                    // snapshot_threshold
        1 +                                    // executed
        1 +                                    // cancelled
        8 +                                    // created_at
        8 +                                    // expires_at
        1 +                                    // approved
        8 +                                    // approved_at
        8 +                                    // execution_window
        1 +                                    // bump
        32;                                    // nonce
    
    /// Check if proposal has expired
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time > self.expires_at
    }
    
    /// Check if swap execution window has expired
    pub fn is_execution_window_expired(&self, current_time: i64) -> bool {
        if !self.approved {
            return false;
        }
        current_time > self.approved_at + self.execution_window
    }
    
    /// Check if proposal can be executed (uses snapshot_threshold)
    pub fn can_execute(&self, current_time: i64, _threshold: u8) -> bool {
        if self.executed || self.cancelled {
            return false;
        }
        
        if self.is_expired(current_time) {
            return false;
        }
        
        // Use snapshot_threshold instead of current threshold
        if self.votes_for < self.snapshot_threshold {
            return false;
        }
        
        // For swaps, must be within execution window
        if self.action.requires_execution_window() {
            if !self.approved {
                return false;
            }
            if self.is_execution_window_expired(current_time) {
                return false;
            }
        }
        
        true
    }
}
