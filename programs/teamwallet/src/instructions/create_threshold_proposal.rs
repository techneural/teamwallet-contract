use anchor_lang::prelude::*;
use crate::state::{TeamWallet, ThresholdProposal};
use crate::errors::TeamWalletError;

/// Creates an on-chain ThresholdProposal account.
/// Only the team wallet owner can call this.
/// Voting is collected off-chain (DB); once enough votes are gathered
/// the owner calls execute_threshold_proposal to apply the change on-chain.
pub fn create_threshold_proposal(
    ctx: Context<CreateThresholdProposal>,
    new_threshold: u8,
    _nonce: Pubkey, // used only in seeds via #[instruction] — stored on proposal
) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;
    let proposal = &mut ctx.accounts.threshold_proposal;

    require!(
        new_threshold >= 1,
        TeamWalletError::InvalidThreshold
    );

    require!(
        new_threshold <= team_wallet.voter_count,
        TeamWalletError::InvalidThreshold
    );

    proposal.team_wallet = team_wallet.key();
    proposal.proposer = ctx.accounts.owner.key();
    proposal.new_threshold = new_threshold;
    proposal.old_threshold = team_wallet.vote_threshold;
    proposal.executed = false;
    proposal.bump = ctx.bumps.threshold_proposal;
    proposal.nonce = _nonce;

    msg!(
        "ThresholdProposal created: {} -> {} / {}",
        team_wallet.vote_threshold,
        new_threshold,
        team_wallet.voter_count
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(new_threshold: u8, nonce: Pubkey)]
pub struct CreateThresholdProposal<'info> {
    #[account(
        init,
        payer = owner,
        space = ThresholdProposal::SPACE,
        seeds = [
            b"threshold_proposal",
            team_wallet.key().as_ref(),
            nonce.as_ref(),
        ],
        bump
    )]
    pub threshold_proposal: Account<'info, ThresholdProposal>,

    #[account(
        has_one = owner,
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}