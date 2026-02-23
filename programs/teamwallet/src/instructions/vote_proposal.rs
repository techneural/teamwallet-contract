use anchor_lang::prelude::*;
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;

pub fn vote_proposal(ctx: Context<VoteProposal>, vote_for: bool) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;

    // Find voter's index in snapshot_voters
    let voter_index = proposal
        .snapshot_voters
        .iter()
        .position(|k| k == &ctx.accounts.voter.key())
        .ok_or(TeamWalletError::NotAuthorizedToVote)? as u8;

    // Check not already voted using index
    require!(
        !proposal.voters_voted.contains(&voter_index),
        TeamWalletError::AlreadyVoted
    );

    require!(
        !proposal.executed,
        TeamWalletError::ProposalAlreadyExecuted
    );

    if vote_for {
        proposal.votes_for += 1;
    } else {
        proposal.votes_against += 1;
    }

    // Store index (1 byte) instead of full pubkey (32 bytes)
    proposal.voters_voted.push(voter_index);

    msg!("Vote recorded from: {} (index: {})", ctx.accounts.voter.key(), voter_index);
    Ok(())
}

#[derive(Accounts)]
pub struct VoteProposal<'info> {
    #[account(
        mut,
        realloc = Proposal::SPACE,
        realloc::payer = voter,
        realloc::zero = false,
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}
