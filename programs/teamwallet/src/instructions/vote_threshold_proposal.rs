use anchor_lang::prelude::*;
use crate::state::{TeamWallet, ThresholdProposal};
use crate::errors::TeamWalletError;

/// Vote on a threshold-change proposal.
/// 
/// Any snapshotted voter (owner or team wallet voter at creation time) may call
/// this once.  After every reject vote the instruction checks whether approval
/// is still mathematically reachable; if not it auto-cancels the proposal so
/// the owner does **not** need to cancel manually.
pub fn vote_threshold_proposal(
    ctx: Context<VoteThresholdProposal>,
    vote_for: bool,
) -> Result<()> {
    let proposal = &mut ctx.accounts.threshold_proposal;
    let voter_key = ctx.accounts.voter.key();

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyExecuted);

    // Voter must be in the snapshot taken at creation time
    require!(
        proposal.snapshot_voters.contains(&voter_key),
        TeamWalletError::NotAuthorizedToVote
    );

    // Prevent double-voting
    require!(
        !proposal.voters_voted.contains(&voter_key),
        TeamWalletError::AlreadyVoted
    );

    proposal.voters_voted.push(voter_key);

    if vote_for {
        proposal.votes_for = proposal.votes_for.saturating_add(1);
        msg!("Vote FOR threshold proposal from {}", voter_key);
    } else {
        proposal.votes_against = proposal.votes_against.saturating_add(1);
        msg!("Vote AGAINST threshold proposal from {}", voter_key);

        // Auto-cancel check: total voters who have NOT yet voted
        let total_voters = proposal.snapshot_voters.len() as u8;
        let votes_cast = proposal.voters_voted.len() as u8;
        let remaining_voters = total_voters.saturating_sub(votes_cast);

        // Even if every remaining voter approves, can we still reach threshold?
        let max_possible_approvals = proposal.votes_for.saturating_add(remaining_voters);
        if max_possible_approvals < proposal.old_threshold {
            proposal.cancelled = true;
            msg!(
                "ThresholdProposal auto-cancelled: max possible approvals ({}) < threshold ({})",
                max_possible_approvals,
                proposal.old_threshold
            );
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct VoteThresholdProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"threshold_proposal",
            team_wallet.key().as_ref(),
            threshold_proposal.nonce.as_ref(),
        ],
        bump = threshold_proposal.bump,
        constraint = threshold_proposal.team_wallet == team_wallet.key()
    )]
    pub threshold_proposal: Account<'info, ThresholdProposal>,

    #[account(
        seeds = [
            b"team_wallet",
            team_wallet.owner.as_ref(),
            team_wallet.name.as_bytes(),
        ],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub voter: Signer<'info>,
}