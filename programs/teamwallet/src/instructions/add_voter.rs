use anchor_lang::prelude::*;
use crate::state::TeamWallet;
use crate::errors::TeamWalletError;

pub fn add_voter(ctx: Context<AddVoter>, voter_pubkey: Pubkey) -> Result<()> {
    let team_wallet = &mut ctx.accounts.team_wallet;
    
    require!(
        team_wallet.voters.len() < 15,
        TeamWalletError::MaxVotersReached
    );
    
    require!(
        !team_wallet.voters.contains(&voter_pubkey)  ,
        TeamWalletError::VoterAlreadyExists
    );
    
    team_wallet.voters.push(voter_pubkey);
    team_wallet.voter_count += 1;
    
    msg!("Voter added: {}", voter_pubkey);
    Ok(())
}

#[derive(Accounts)]
pub struct AddVoter<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    pub owner: Signer<'info>,
}
