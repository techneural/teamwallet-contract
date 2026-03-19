use anchor_lang::prelude::*;
use crate::state::{TeamWallet, TokenProposal};
use crate::errors::TeamWalletError;

pub fn vote_token_proposal(ctx: Context<VoteTokenProposal>, vote_for: bool) -> Result<()> {
    let proposal = &mut ctx.accounts.token_proposal;
    
    let clock = Clock::get()?;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        !proposal.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalExpired
    );

    let voter_index = proposal
        .snapshot_voters
        .iter()
        .position(|k| k == &ctx.accounts.voter.key())
        .ok_or(TeamWalletError::NotAuthorizedToVote)? as u8;

    require!(
        !proposal.voters_voted.contains(&voter_index),
        TeamWalletError::AlreadyVoted
    );
    
    if vote_for {
        proposal.votes_for = proposal.votes_for.saturating_add(1);
    } else {
        proposal.votes_against = proposal.votes_against.saturating_add(1);
    }

    proposal.voters_voted.push(voter_index);

    msg!("Token proposal vote recorded from: {} (index: {})", ctx.accounts.voter.key(), voter_index);
    Ok(())
}

#[derive(Accounts)]
pub struct VoteTokenProposal<'info> {
    #[account(mut)]
    pub token_proposal: Account<'info, TokenProposal>,
    
    #[account(
        constraint = token_proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}
