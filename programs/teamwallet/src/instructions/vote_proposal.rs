use anchor_lang::prelude::*;
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;

pub fn vote_proposal(ctx: Context<VoteProposal>, vote_for: bool) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    
    let is_voter = team_wallet.voters.contains(&ctx.accounts.voter.key());
    let is_contributor = team_wallet.contributors.contains(&ctx.accounts.voter.key());
    let is_owner = team_wallet.owner == ctx.accounts.voter.key();
    
    require!(
        is_voter || is_contributor || is_owner,
        TeamWalletError::NotAuthorizedToVote
    );
    
    require!(
        !proposal.voters_voted.contains(&ctx.accounts.voter.key()),
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
    
    proposal.voters_voted.push(ctx.accounts.voter.key());
    
    msg!("Vote recorded from: {}", ctx.accounts.voter.key());
    Ok(())
}

#[derive(Accounts)]
pub struct VoteProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    pub voter: Signer<'info>,
}