use anchor_lang::prelude::*;
use crate::state::{TeamWallet, ThresholdProposal};
use crate::errors::TeamWalletError;


pub fn create_threshold_proposal(
    ctx: Context<CreateThresholdProposal>,
    new_threshold: u8,
    _nonce: Pubkey, 
) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;
    let proposal = &mut ctx.accounts.threshold_proposal;
    let owner_key = ctx.accounts.owner.key();

    require!(
        new_threshold >= 1,
        TeamWalletError::InvalidThreshold
    );

    require!(
        new_threshold <= team_wallet.voter_count,
        TeamWalletError::InvalidThreshold
    );

    // Snapshot all eligible voters (owner + voters list)
    let mut snapshot: Vec<Pubkey> = vec![owner_key];
    for v in &team_wallet.voters {
        if *v != owner_key {
            snapshot.push(*v);
        }
    }

    proposal.team_wallet = team_wallet.key();
    proposal.proposer = owner_key;
    proposal.new_threshold = new_threshold;
    proposal.old_threshold = team_wallet.vote_threshold;
    proposal.executed = false;
    proposal.cancelled = false;
    proposal.bump = ctx.bumps.threshold_proposal;
    proposal.nonce = _nonce;
    proposal.snapshot_voters = snapshot;
    // Owner auto-approves on creation
    proposal.voters_voted = vec![owner_key];
    proposal.votes_for = 1;
    proposal.votes_against = 0;

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
        space = ThresholdProposal::SPACE, // updated SPACE includes voters vecs
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