use anchor_lang::prelude::*;

#[error_code]
pub enum TeamWalletError {
    #[msg("Maximum number of voters reached (10)")]
    MaxVotersReached,
    
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
    
    #[msg("Invalid proposal type")]
    InvalidProposalType,
    
    #[msg("Invalid mint address")]
    InvalidMint,
    
    #[msg("Maximum number of contributors reached (10)")]
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
}