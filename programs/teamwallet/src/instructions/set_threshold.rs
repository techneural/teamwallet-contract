use anchor_lang::prelude::*;
use crate::state::TeamWallet;
use crate::errors::TeamWalletError;


pub fn set_threshold(ctx: Context<SetThreshold>, new_threshold: u8) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;

    require!(
        new_threshold >= 1,
        TeamWalletError::InvalidThreshold
    );

    require!(
        new_threshold <= team_wallet.voter_count,
        TeamWalletError::InvalidThreshold
    );

    team_wallet.vote_threshold = new_threshold;

    msg!(
        "Threshold updated to {} / {}",
        new_threshold,
        team_wallet.voter_count
    );
    Ok(())
}

#[derive(Accounts)]
pub struct SetThreshold<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    pub owner: Signer<'info>,
}