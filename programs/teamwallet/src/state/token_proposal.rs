use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TokenAction {
    Mint,
    Burn,
    FreezeAccount,
    ThawAccount,
    SetMintAuthority,
    SetFreezeAuthority,
    UpdateMetadata,
    SetTransferFee,
    WithdrawTransferFees,
    EnableConfidentialTransfers,
    DisableConfidentialTransfers,
    UpdateInterestRate,
    SetPermanentDelegate,
    UpdateGroupPointer,
    UpdateMemberPointer,
    Transfer,
}

#[account]
pub struct TokenProposal {
    pub proposal_id: Pubkey,
    pub team_wallet: Pubkey,
    pub proposer: Pubkey,
    pub mint: Pubkey,
    pub action: TokenAction,
    pub amount: u64,
    pub recipient: Option<Pubkey>,
    pub metadata: Option<TokenMetadataParams>,
    pub transfer_fee_config: Option<TransferFeeParams>,
    pub interest_rate: Option<i16>,
    pub votes_for: u8,
    pub votes_against: u8,
    pub voters_voted: Vec<u8>,
    pub executed: bool,
    pub cancelled: bool,
    pub bump: u8,
    pub snapshot_voters: Vec<Pubkey>,
    
    // Expiry fields
    pub created_at: i64,
    pub expires_at: i64,
}

impl TokenProposal {
    pub const MAX_VOTERS: usize = 15;
    pub const MAX_CONTRIBUTORS: usize = 15;
    pub const MAX_SNAPSHOT: usize = Self::MAX_VOTERS + Self::MAX_CONTRIBUTORS;
    
    /// Default expiry: 7 days
    pub const DEFAULT_EXPIRY: i64 = 7 * 24 * 60 * 60;

    pub const SPACE: usize =
        8 +                                // discriminator
        32 +                               // proposal_id
        32 +                               // team_wallet
        32 +                               // proposer
        32 +                               // mint
        1 +                                // action enum
        8 +                                // amount
        33 +                               // recipient Option<Pubkey>
        255 +                              // metadata Option<TokenMetadataParams>
        11 +                               // transfer_fee_config Option<TransferFeeParams>
        3 +                                // interest_rate Option<i16>
        1 +                                // votes_for
        1 +                                // votes_against
        4 + Self::MAX_SNAPSHOT +           // voters_voted Vec<u8>
        1 +                                // executed
        1 +                                // cancelled
        1 +                                // bump
        4 + (32 * Self::MAX_SNAPSHOT) +    // snapshot_voters Vec<Pubkey>
        8 +                                // created_at
        8;                                 // expires_at
        
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time > self.expires_at
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenMetadataParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferFeeParams {
    pub transfer_fee_basis_points: u16,
    pub maximum_fee: u64,
}
