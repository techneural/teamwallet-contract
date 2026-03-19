use anchor_lang::prelude::*;
use crate::state::{TeamWallet, SwapIntent};
use crate::errors::TeamWalletError;

pub fn vote_swap_intent(ctx: Context<VoteSwapIntent>, vote_for: bool) -> Result<()> {
    let intent = &mut ctx.accounts.swap_intent;
    let team_wallet = &ctx.accounts.team_wallet;
    let voter = &ctx.accounts.voter;
    
    let clock = Clock::get()?;
    
    require!(!intent.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!intent.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        !intent.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalExpired
    );
    
    let voter_index = intent
        .snapshot_voters
        .iter()
        .position(|k| k == &voter.key())
        .ok_or(TeamWalletError::NotAuthorizedToVote)? as u8;
    
    require!(
        !intent.voters_voted.contains(&voter_index),
        TeamWalletError::AlreadyVoted
    );
    
    if vote_for {
        intent.votes_for = intent.votes_for.saturating_add(1);
    } else {
        intent.votes_against = intent.votes_against.saturating_add(1);
    }
    intent.voters_voted.push(voter_index);
    
    if !intent.approved && intent.votes_for >= team_wallet.vote_threshold {
        intent.approved = true;
        intent.approved_at = clock.unix_timestamp;
        msg!("Swap intent APPROVED! Execute within {} seconds", intent.execution_window_seconds);
    }
    
    msg!("Vote: {} voted {}", voter.key(), if vote_for { "FOR" } else { "AGAINST" });
    
    Ok(())
}

#[derive(Accounts)]
pub struct VoteSwapIntent<'info> {
    #[account(
        mut,
        seeds = [
            b"swap_intent",
            team_wallet.key().as_ref(),
            swap_intent.nonce.as_ref(),
        ],
        bump = swap_intent.bump,
        constraint = swap_intent.team_wallet == team_wallet.key()
    )]
    pub swap_intent: Account<'info, SwapIntent>,
    
    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    #[account(mut)]
    pub voter: Signer<'info>,
}
