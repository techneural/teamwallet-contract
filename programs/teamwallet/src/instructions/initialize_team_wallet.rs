use anchor_lang::prelude::*;
use crate::state::TeamWallet;
use crate::errors::TeamWalletError;

pub fn initialize_team_wallet(
    ctx: Context<InitializeTeamWallet>,
    name: String,
    vote_threshold: u8,
    voters: Vec<Pubkey>,
) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;
    let owner_key = ctx.accounts.owner.key();

    // Validate voters count
    require!(
        voters.len() <= 14,
        TeamWalletError::MaxVotersReached
    );

    // Check for duplicates
    let mut unique_voters = voters.clone();
    unique_voters.sort();
    unique_voters.dedup();
    require!(
        unique_voters.len() == voters.len(),
        TeamWalletError::DuplicateVoter
    );

    // Owner can't be in voters list (auto-added)
    require!(
        !voters.contains(&owner_key),
        TeamWalletError::OwnerInMembersList
    );

    // Validate threshold
    let total_voters = (voters.len() as u8) + 1; // +1 for owner
    require!(
        vote_threshold >= 1 && vote_threshold <= total_voters,
        TeamWalletError::InvalidThreshold
    );

    // Set team wallet fields
    team_wallet.owner = owner_key;
    team_wallet.name = name;
    team_wallet.vote_threshold = vote_threshold;

    // Add owner + voters
    let mut all_voters = vec![owner_key];
    all_voters.extend(voters);
    team_wallet.voters = all_voters;
    team_wallet.voter_count = team_wallet.voters.len() as u8;

    team_wallet.contributors = vec![];
    team_wallet.proposal_count = 0;
    team_wallet.bump = ctx.bumps.team_wallet;

    msg!("Team wallet initialized: {} voters, threshold {}", 
        team_wallet.voter_count, vote_threshold);

    Ok(())
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitializeTeamWallet<'info> {
    #[account(
        init,
        payer = owner,
        space = TeamWallet::MAX_SIZE,
        seeds = [b"team_wallet", owner.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}
