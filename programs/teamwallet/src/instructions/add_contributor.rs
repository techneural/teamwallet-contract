use anchor_lang::prelude::*;
use crate::state::TeamWallet;
use crate::errors::TeamWalletError;

pub fn add_contributor(ctx: Context<AddContributor>, contributor_pubkey: Pubkey) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;
    
    require!(
        team_wallet.contributors.len() < 10,
        TeamWalletError::MaxContributorsReached
    );
    
    require!(
        !team_wallet.contributors.contains(&contributor_pubkey),
        TeamWalletError::ContributorAlreadyExists
    );
    
    require!(
        !team_wallet.voters.contains(&contributor_pubkey),
        TeamWalletError::AlreadyAVoter
    );
    
    team_wallet.contributors.push(contributor_pubkey);
    
    msg!("Contributor added: {}", contributor_pubkey);
    Ok(())
}

#[derive(Accounts)]
pub struct AddContributor<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    pub owner: Signer<'info>,
}