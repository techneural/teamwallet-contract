use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TokenAction {
    Mint,                      // Mint new tokens
    Burn,                      // Burn tokens
    FreezAccount,              // Freeze a token account
    ThawAccount,               // Unfreeze a token account
    SetAuthority,              // Transfer mint/freeze authority
    // Token Extensions (Token-2022)
    UpdateMetadata,            // Update token metadata
    SetTransferFee,            // Set transfer fee basis points
    WithdrawTransferFees,      // Withdraw accumulated transfer fees
    EnableConfidentialTransfers, // Enable confidential transfers on account
    DisableConfidentialTransfers, // Disable confidential transfers
    UpdateInterestRate,        // Update interest bearing rate
    SetPermanentDelegate,      // Set permanent delegate
    UpdateGroupPointer,        // Update token group pointer
    UpdateMemberPointer,       // Update token member pointer
}

#[account]
pub struct TokenProposal {
    pub proposal_id : Pubkey,
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
    pub voters_voted: Vec<Pubkey>,
    pub executed: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenMetadataParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferFeeParams {
    pub transfer_fee_basis_points: u16,  // Fee in basis points (1 = 0.01%)
    pub maximum_fee: u64,                 // Maximum fee amount
}
