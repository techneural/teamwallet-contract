use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;
 
pub fn execute_proposal_sol(ctx: Context<ExecuteProposalSol>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;
   
    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.is_token_transfer, TeamWalletError::InvalidProposalType);
   
    let votes_needed = ((team_wallet.voter_count as f64) * (team_wallet.vote_threshold as f64 / 100.0)).ceil() as u8;
   
   
    require!(
        proposal.votes_for >= votes_needed,
        TeamWalletError::InsufficientVotes
    );
   
    **ctx.accounts.team_wallet.to_account_info().try_borrow_mut_lamports()? -= proposal.amount;
    **ctx.accounts.recipient.try_borrow_mut_lamports()? += proposal.amount;
   
    proposal.executed = true;
   
    msg!("SOL transfer executed: {} lamports to {}", proposal.amount, proposal.recipient);
    Ok(())
}
 
pub fn execute_proposal_token(ctx: Context<ExecuteProposalToken>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;
   
    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(proposal.is_token_transfer, TeamWalletError::InvalidProposalType);
   
    let votes_needed = ((team_wallet.voter_count as f64) * (team_wallet.vote_threshold as f64 / 100.0)).ceil() as u8;
    
    require!(
        proposal.votes_for >= votes_needed,
        TeamWalletError::InsufficientVotes
    );
   
    // Verify the mint matches
    require!(
        ctx.accounts.team_token_account.mint == proposal.mint.unwrap(),
        TeamWalletError::InvalidMint
    );
   
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet",
        team_wallet.owner.as_ref(),
        name_bytes,
        &[team_wallet.bump],
    ];
    let signer_seeds = &[&seeds[..]];
   
    let cpi_accounts = Transfer {
        from: ctx.accounts.team_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.team_wallet.to_account_info(),
    };
   
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
   
    token::transfer(cpi_ctx, proposal.amount)?;
   
    proposal.executed = true;
   
    msg!("Token transfer executed: {} tokens to {}", proposal.amount, proposal.recipient);
    Ok(())
}
 
#[derive(Accounts)]
pub struct ExecuteProposalSol<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
   
    #[account(
        mut,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,
   
    /// CHECK: This is the recipient of the transfer
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
   
    pub executor: Signer<'info>,
}
 
#[derive(Accounts)]
pub struct ExecuteProposalToken<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
   
    #[account(
        mut,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,
   
    #[account(mut)]
    pub team_token_account: Account<'info, TokenAccount>,
   
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
   
    pub token_program: Program<'info, Token>,
   
    pub executor: Signer<'info>,
}
 
 