use anchor_lang::prelude::*;
use crate::state::TeamWallet;
use crate::errors::TeamWalletError;

pub fn remove_contributor(ctx: Context<RemoveContributor>, contributor_pubkey: Pubkey) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;
    
    require!(
        team_wallet.contributors.contains(&contributor_pubkey),
        TeamWalletError::ContributorNotFound
    );
    
    team_wallet.contributors.retain(|&c| c != contributor_pubkey);
    
    msg!("Contributor removed: {}", contributor_pubkey);
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveContributor<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    pub owner: Signer<'info>,
}

