use anchor_lang::prelude::*;

#[error_code]
pub enum TeamWalletError {
    #[msg("Maximum number of voters reached (15)")]
    MaxVotersReached,

    #[msg("Authority missing")]
    FreezeAuthorityMissing,

    #[msg("Invalid authority type")]
    InvalidAuthorityType,
    
    #[msg("Voter already exists")]
    VoterAlreadyExists,
    
    #[msg("Voter not found")]
    VoterNotFound,
    
    #[msg("Cannot remove the owner")]
    CannotRemoveOwner,
    
    #[msg("Not a voter in this team wallet")]
    NotAVoter,
    
    #[msg("Already voted on this proposal")]
    AlreadyVoted,
    
    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,
    
    #[msg("Insufficient votes to execute proposal")]
    InsufficientVotes,

    #[msg("Invalid upgrade buffer")]
    InvalidUpgradeBuffer,

    #[msg("Invalid program data account")]
    InvalidProgramData,
    
    #[msg("Invalid proposal type")]
    InvalidProposalType,
    
    #[msg("Invalid mint address")]
    InvalidMint,
    
    #[msg("Maximum number of contributors reached (15)")]
    MaxContributorsReached,
    
    #[msg("Contributor already exists")]
    ContributorAlreadyExists,
    
    #[msg("Contributor not found")]
    ContributorNotFound,
    
    #[msg("User is already a voter")]
    AlreadyAVoter,
    
    #[msg("Not a voter or contributor in this team wallet")]
    NotAVoterOrContributor,
    
    #[msg("Not authorized to vote on proposals")]
    NotAuthorizedToVote,
    
    #[msg("Mint address is required for token transfers")]
    MintRequired,
    
    #[msg("Duplicate voter in initialization list")]
    DuplicateVoter,
    
    #[msg("Owner cannot be in voters list (owner is automatically a voter)")]
    OwnerInMembersList,

    #[msg("Recipient is required for this token action")]
    RecipientRequired,
    
    #[msg("Metadata is required for this action")]
    MetadataRequired,
    
    #[msg("Transfer fee configuration is required")]
    TransferFeeConfigRequired,
    
    #[msg("Interest rate is required for this action")]
    InterestRateRequired,

    #[msg("Invalid remaining accounts")]
    InvalidRemainingAccounts,

    #[msg("Invalid route data")]
    InvalidRouteData,

    #[msg("Invalid proposal data")]
    InvalidProposalData,

    #[msg("Input mint required for swap")]
    InputMintRequired,

    #[msg("Output mint required for swap")]
    OutputMintRequired,

    #[msg("Minimum output amount required")]
    MinOutputRequired,

    #[msg("Invalid minimum output amount")]
    InvalidMinOutput,

    #[msg("Cannot swap same mint")]
    SameMintSwap,

    #[msg("Slippage too high (max 50%)")]
    SlippageTooHigh,

    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Swap failed")]
    SwapFailed,

    #[msg("Slippage exceeded")]
    SlippageExceeded,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Invalid threshold value")]
    InvalidThreshold,

    #[msg("Proposal has been cancelled")]
    ProposalAlreadyCancelled,

    #[msg("Proposal has expired")]
    ProposalExpired,

    #[msg("Proposal has not expired yet")]
    ProposalNotExpired,

    #[msg("Proposal has not been executed")]
    ProposalNotExecuted,

    #[msg("Not authorized to cancel")]
    NotAuthorizedToCancel,

    #[msg("Swap execution window expired")]
    SwapExecutionWindowExpired,
}
