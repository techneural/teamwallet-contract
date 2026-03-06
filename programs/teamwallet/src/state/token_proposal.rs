use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TokenAction {
    Mint,
    Burn,
    FreezAccount,
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
    Transfer, // ← NEW: send tokens to a friend wallet
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
    // Stores the INDEX of each voter in snapshot_voters — 1 byte per member
    pub voters_voted: Vec<u8>,
    pub executed: bool,
    pub bump: u8,
    // snapshot_voters = voters(max 15) + contributors(max 15) = 30 max
    // NOTE: owner is already inside voters[], do NOT push owner separately
    pub snapshot_voters: Vec<Pubkey>,
}

impl TokenProposal {
    pub const MAX_VOTERS: usize = 15;
    pub const MAX_CONTRIBUTORS: usize = 15;
    pub const MAX_SNAPSHOT: usize = Self::MAX_VOTERS + Self::MAX_CONTRIBUTORS; // 30

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
        4 + Self::MAX_SNAPSHOT +           // voters_voted Vec<u8> → 1 byte per member (30 max)
        1 +                                // executed
        1 +                                // bump
        4 + (32 * Self::MAX_SNAPSHOT);     // snapshot_voters Vec<Pubkey> → 30 entries
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