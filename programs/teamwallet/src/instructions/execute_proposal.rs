use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
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

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.team_token_account.to_account_info(),
        mint: ctx.accounts.token_mint.to_account_info(),       
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.team_wallet.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    token_interface::transfer_checked(
        cpi_ctx,
        proposal.amount,
        ctx.accounts.token_mint.decimals,  
    )?;
   
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
    pub team_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_mint: InterfaceAccount<'info, Mint>,                

    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
   
    pub token_program: Interface<'info, TokenInterface>,  
   
    pub executor: Signer<'info>,
}