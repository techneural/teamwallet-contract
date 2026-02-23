use anchor_lang::prelude::*;
use crate::{errors::TeamWalletError, state::TeamWallet};

pub fn initialize_team_wallet(
    ctx: Context<InitializeTeamWallet>,
    name: String,
    vote_threshold: u8,
    voters: Vec<Pubkey>, 
) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;

        msg!("--- Initializing team wallet ---");


    // Validate limits (14 additional voters max, plus owner = 11 total max)
    require!(
        voters.len() <= 14,  // max 14 + owner = 15 total, consistent with add_voter limit
        TeamWalletError::MaxVotersReached
        
    );

    // Check for duplicates in voters list
    let mut unique_voters = voters.clone();
    unique_voters.sort();
    unique_voters.dedup();
    require!(
        unique_voters.len() == voters.len(),
        TeamWalletError::DuplicateVoter
    );
    
    // Check if owner is in voters list (owner is automatically added)
    let owner_key = ctx.accounts.owner.key();
    require!(
        !voters.contains(&owner_key),
        TeamWalletError::OwnerInMembersList
    );

    team_wallet.owner = owner_key;
    team_wallet.name = name;
    team_wallet.vote_threshold = vote_threshold;


   // Owner is always the first voter
    let mut all_voters = vec![owner_key];
    all_voters.extend(voters);
    team_wallet.voters = all_voters;
    team_wallet.voter_count = team_wallet.voters.len() as u8;

    team_wallet.contributors = vec![];
    team_wallet.bump = ctx.bumps.team_wallet;
    
    msg!("Team wallet initialized by owner: {}", ctx.accounts.owner.key());
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


