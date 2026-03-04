use anchor_lang::prelude::*;
use crate::state::{TeamWallet, ThresholdProposal};
use crate::errors::TeamWalletError;

/// Executes a ThresholdProposal on-chain:
/// - Validates it hasn't been executed yet
/// - Updates team_wallet.vote_threshold to the proposed value
/// - Marks the proposal as executed
/// - Closes the proposal account and returns rent to owner
///
/// The off-chain vote collection is done via DB APIs.
/// The owner calls this once votesFor >= currentThreshold in the DB.
pub fn execute_threshold_proposal(
    ctx: Context<ExecuteThresholdProposal>,
    _nonce: Pubkey, // used in seeds derivation via #[instruction]
) -> Result<()> {
    let proposal = &mut ctx.accounts.threshold_proposal;
    let team_wallet = &mut ctx.accounts.team_wallet;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);

    require!(
        proposal.new_threshold >= 1,
        TeamWalletError::InvalidThreshold
    );

    require!(
        proposal.new_threshold <= team_wallet.voter_count,
        TeamWalletError::InvalidThreshold
    );

    // Apply the threshold change
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
        close = owner, // return rent lamports to owner on close
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