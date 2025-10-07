use anchor_lang::prelude::*;
use crate::state::TeamWallet;
use crate::errors::TeamWalletError;

pub fn remove_voter(ctx: Context<RemoveVoter>, voter_pubkey: Pubkey) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;
    
    require!(
        team_wallet.voters.contains(&voter_pubkey),
        TeamWalletError::VoterNotFound
    );
    
    require!(
        voter_pubkey != team_wallet.owner,
        TeamWalletError::CannotRemoveOwner
    );
    
    team_wallet.voters.retain(|&v| v != voter_pubkey);
    team_wallet.voter_count -= 1;
    
    msg!("Voter removed: {}", voter_pubkey);
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveVoter<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    pub owner: Signer<'info>,
}