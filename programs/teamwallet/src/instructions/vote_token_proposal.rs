use anchor_lang::prelude::*;
use crate::state::{TeamWallet, TokenProposal};
use crate::errors::TeamWalletError;
pub fn vote_token_proposal(ctx: Context<VoteTokenProposal>, vote_for: bool) -> Result<()> {
    let proposal = &mut ctx.accounts.token_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    
    require!(
        proposal.snapshot_voters.contains(&ctx.accounts.voter.key()),
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
    
    msg!("Token proposal vote recorded from: {}", ctx.accounts.voter.key());
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
    
    pub voter: Signer<'info>,
}