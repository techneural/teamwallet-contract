use anchor_lang::prelude::*;
use crate::state::{TeamWallet, ThresholdProposal};
use crate::errors::TeamWalletError;

pub fn vote_threshold_proposal(
    ctx: Context<VoteThresholdProposal>,
    vote_for: bool,
) -> Result<()> {
    let proposal = &mut ctx.accounts.threshold_proposal;
    let voter_key = ctx.accounts.voter.key();
    
    let clock = Clock::get()?;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        !proposal.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalExpired
    );

    require!(
        proposal.snapshot_voters.contains(&voter_key),
        TeamWalletError::NotAuthorizedToVote
    );

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

        // Auto-cancel check
        let total_voters = proposal.snapshot_voters.len() as u8;
        let votes_cast = proposal.voters_voted.len() as u8;
        let remaining_voters = total_voters.saturating_sub(votes_cast);
        let max_possible_approvals = proposal.votes_for.saturating_add(remaining_voters);
        
        if max_possible_approvals < proposal.old_threshold {
            proposal.cancelled = true;
            msg!(
                "ThresholdProposal auto-cancelled: max possible ({}) < threshold ({})",
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
