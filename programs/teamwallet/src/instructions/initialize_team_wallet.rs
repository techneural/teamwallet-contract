use anchor_lang::prelude::*;
use crate::state::TeamWallet;

pub fn initialize_team_wallet(
    ctx: Context<InitializeTeamWallet>,
    name: String,
    vote_threshold: u8,
) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;
    team_wallet.owner = ctx.accounts.owner.key();
    team_wallet.name = name;
    team_wallet.vote_threshold = vote_threshold;
    team_wallet.voter_count = 1;
    team_wallet.voters = vec![ctx.accounts.owner.key()];
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
        space = 8 + 32 + 36 + 1 + 1 + 324 + 1,
        seeds = [b"team_wallet", owner.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}