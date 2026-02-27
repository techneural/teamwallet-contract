
use anchor_lang::prelude::*;
use crate::{errors::TeamWalletError, state::TeamWallet};

pub fn initialize_team_wallet(
    ctx: Context<InitializeTeamWallet>,
    name: String,
    vote_threshold: u8,
    voters: Vec<Pubkey>,
    lookup_table: Pubkey,     
) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;

    msg!("--- Initializing team wallet ---");

    require!(
        voters.len() <= 14,
        TeamWalletError::MaxVotersReached
    );

    let mut unique_voters = voters.clone();
    unique_voters.sort();
    unique_voters.dedup();
    require!(
        unique_voters.len() == voters.len(),
        TeamWalletError::DuplicateVoter
    );

    let owner_key = ctx.accounts.owner.key();
    require!(
        !voters.contains(&owner_key),
        TeamWalletError::OwnerInMembersList
    );

    team_wallet.owner = owner_key;
    team_wallet.name = name;
    team_wallet.vote_threshold = vote_threshold;
    team_wallet.lookup_table = lookup_table; 

    let mut all_voters = vec![owner_key];
    all_voters.extend(voters);
    team_wallet.voters = all_voters;
    team_wallet.voter_count = team_wallet.voters.len() as u8;

    team_wallet.contributors = vec![];
    team_wallet.bump = ctx.bumps.team_wallet;

    msg!("Team wallet initialized by owner: {}", ctx.accounts.owner.key());
    msg!("Lookup table: {}", lookup_table);
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