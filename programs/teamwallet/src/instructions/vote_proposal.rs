use anchor_lang::prelude::*;
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;

/// Vote on any proposal type
pub fn vote_proposal(
    ctx: Context<VoteProposal>,
    vote_for: bool,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let _team_wallet = &ctx.accounts.team_wallet;
    let voter = &ctx.accounts.voter;
    
    let clock = Clock::get()?;

    // Check proposal is still voteable
    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        !proposal.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalExpired
    );

    // Find voter in snapshot
    let voter_index = proposal
        .snapshot_voters
        .iter()
        .position(|k| k == &voter.key())
        .ok_or(TeamWalletError::NotAuthorizedToVote)? as u8;

    // Check not already voted
    require!(
        !proposal.voters_voted.contains(&voter_index),
        TeamWalletError::AlreadyVoted
    );

    // Record vote
    if vote_for {
        proposal.votes_for = proposal.votes_for.saturating_add(1);
    } else {
        proposal.votes_against = proposal.votes_against.saturating_add(1);
    }
    proposal.voters_voted.push(voter_index);

    // Check if threshold now reached (for swap execution window)
    // Use snapshot_threshold for consistency
    if !proposal.approved && proposal.votes_for >= proposal.snapshot_threshold {
        proposal.approved = true;
        proposal.approved_at = clock.unix_timestamp;
        
        if proposal.action.requires_execution_window() {
            msg!("Swap approved! Execution window: {} seconds", proposal.execution_window);
        }
    }

    // Auto-cancel check: if remaining voters can't reach snapshot_threshold
    let total_voters = proposal.snapshot_voters.len() as u8;
    let votes_cast = proposal.voters_voted.len() as u8;
    let remaining = total_voters.saturating_sub(votes_cast);
    let max_possible = proposal.votes_for.saturating_add(remaining);
    
    // Use snapshot_threshold (not current team_wallet.vote_threshold)
    // This ensures correct auto-cancel even if threshold changed after proposal creation
    if max_possible < proposal.snapshot_threshold {
        proposal.cancelled = true;
        msg!("Proposal auto-cancelled: cannot reach threshold ({} < {})", 
            max_possible, proposal.snapshot_threshold);
    }

    msg!("Vote recorded: {} (for: {}, against: {}, threshold: {})", 
        if vote_for { "FOR" } else { "AGAINST" },
        proposal.votes_for,
        proposal.votes_against,
        proposal.snapshot_threshold
    );

    Ok(())
}

#[derive(Accounts)]
pub struct VoteProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"proposal",
            team_wallet.key().as_ref(),
            proposal.nonce.as_ref(),
        ],
        bump = proposal.bump,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub voter: Signer<'info>,
}
