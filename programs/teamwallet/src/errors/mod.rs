use anchor_lang::prelude::*;

#[error_code]
pub enum TeamWalletError {
    // ═══════════════════════════════════════════════════════════════════════
    // TEAM WALLET
    // ═══════════════════════════════════════════════════════════════════════
    
    #[msg("Maximum number of voters reached (15)")]
    MaxVotersReached,

    #[msg("Maximum number of contributors reached (15)")]
    MaxContributorsReached,

    #[msg("Voter already exists")]
    VoterAlreadyExists,

    #[msg("Voter not found")]
    VoterNotFound,

    #[msg("Contributor already exists")]
    ContributorAlreadyExists,

    #[msg("Contributor not found")]
    ContributorNotFound,

    #[msg("Cannot remove the owner")]
    CannotRemoveOwner,

    #[msg("Duplicate voter in initialization list")]
    DuplicateVoter,

    #[msg("Owner cannot be in voters list")]
    OwnerInMembersList,

    #[msg("User is already a voter")]
    AlreadyAVoter,

    #[msg("Invalid threshold value")]
    InvalidThreshold,

    // ═══════════════════════════════════════════════════════════════════════
    // PROPOSAL
    // ═══════════════════════════════════════════════════════════════════════

    #[msg("Not a voter or contributor")]
    NotAVoterOrContributor,

    #[msg("Not authorized to vote")]
    NotAuthorizedToVote,

    #[msg("Already voted on this proposal")]
    AlreadyVoted,

    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,

    #[msg("Proposal has been cancelled")]
    ProposalAlreadyCancelled,

    #[msg("Proposal has expired")]
    ProposalExpired,

    #[msg("Proposal has not expired yet")]
    ProposalNotExpired,

    #[msg("Insufficient votes to execute")]
    InsufficientVotes,

    #[msg("Not authorized to cancel")]
    NotAuthorizedToCancel,

    #[msg("Invalid proposal type for this operation")]
    InvalidProposalType,

    // ═══════════════════════════════════════════════════════════════════════
    // SWAP
    // ═══════════════════════════════════════════════════════════════════════

    #[msg("Cannot swap same mint")]
    SameMintSwap,

    #[msg("Slippage too high (max 50%)")]
    SlippageTooHigh,

    #[msg("Swap execution window expired")]
    SwapExecutionWindowExpired,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Swap not yet approved")]
    SwapNotApproved,

    // ═══════════════════════════════════════════════════════════════════════
    // TOKEN
    // ═══════════════════════════════════════════════════════════════════════

    #[msg("Invalid mint address")]
    InvalidMint,

    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Recipient required")]
    RecipientRequired,

    #[msg("Metadata required")]
    MetadataRequired,

    #[msg("Authority missing")]
    AuthorityMissing,

    // ═══════════════════════════════════════════════════════════════════════
    // PROGRAM UPGRADE/DELETE
    // ═══════════════════════════════════════════════════════════════════════

    #[msg("Invalid upgrade buffer")]
    InvalidUpgradeBuffer,

    #[msg("Invalid program data account")]
    InvalidProgramData,

    // ═══════════════════════════════════════════════════════════════════════
    // GENERAL
    // ═══════════════════════════════════════════════════════════════════════

    #[msg("Invalid remaining accounts")]
    InvalidRemainingAccounts,

    #[msg("Invalid data")]
    InvalidData,

    #[msg("Overflow")]
    Overflow,
}
