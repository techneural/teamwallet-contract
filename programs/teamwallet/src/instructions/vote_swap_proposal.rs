use anchor_lang::prelude::*;
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;

pub fn vote_swap_proposal(
    ctx: Context<VoteSwapProposal>,
    vote_for: bool
) -> Result<()> {

    let wallet = &ctx.accounts.team_wallet;
    let proposal = &mut ctx.accounts.proposal;
    let voter = ctx.accounts.voter.key();

    require!(proposal.is_swap_proposal, TeamWalletError::InvalidProposalType);
    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);

    // Auth check
    require!(
        wallet.owner == voter
            || wallet.voters.contains(&voter)
            || wallet.contributors.contains(&voter),
        TeamWalletError::NotAuthorizedToVote
    );

    // Prevent double vote
   let voter_index = proposal

    .snapshot_voters

    .iter()

    .position(|k| k == &voter)

    .ok_or(TeamWalletError::NotAuthorizedToVote)? as u8;
 
require!(

    !proposal.voters_voted.contains(&voter_index),

    TeamWalletError::AlreadyVoted

);

proposal.voters_voted.push(voter_index);
 
    if vote_for {
        proposal.votes_for += 1;
    } else {
        proposal.votes_against += 1;
    }

    // Check threshold
    let required_votes = ((wallet.voter_count as f64)
        * (wallet.vote_threshold as f64 / 100.0))
        .ceil() as u8;

    if proposal.votes_for >= required_votes {
        proposal.ready_to_execute = true;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct VoteSwapProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    pub voter: Signer<'info>,
}
