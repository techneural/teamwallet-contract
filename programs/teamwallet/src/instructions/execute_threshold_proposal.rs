use anchor_lang::prelude::*;
use crate::state::{TeamWallet, ThresholdProposal};
use crate::errors::TeamWalletError;


pub fn execute_threshold_proposal(
    ctx: Context<ExecuteThresholdProposal>,
    _nonce: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.threshold_proposal;
    let team_wallet = &mut ctx.accounts.team_wallet;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    
    // FIXED: Check if proposal was cancelled
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyExecuted);

    // FIXED: Verify votes meet threshold before execution
    // Uses the old_threshold (threshold at time of proposal creation)
    require!(
        proposal.votes_for >= proposal.old_threshold,
        TeamWalletError::InsufficientVotes
    );

    require!(
        proposal.new_threshold >= 1,
        TeamWalletError::InvalidThreshold
    );

    require!(
        proposal.new_threshold <= team_wallet.voter_count,
        TeamWalletError::InvalidThreshold
    );

    team_wallet.vote_threshold = proposal.new_threshold;
    proposal.executed = true;

    msg!(
        "Threshold executed: {} -> {}",
        proposal.old_threshold,
        proposal.new_threshold
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(nonce: Pubkey)]
pub struct ExecuteThresholdProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"threshold_proposal",
            team_wallet.key().as_ref(),
            nonce.as_ref(),
        ],
        bump = threshold_proposal.bump,
        has_one = team_wallet,
        close = owner, 
    )]
    pub threshold_proposal: Account<'info, ThresholdProposal>,

    #[account(
        mut,
        has_one = owner,
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}
