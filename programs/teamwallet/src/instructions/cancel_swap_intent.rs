use anchor_lang::prelude::*;
use crate::state::{TeamWallet, SwapIntent};
use crate::errors::TeamWalletError;

/// Cancel a swap intent (proposer or owner only)
pub fn cancel_swap_intent(ctx: Context<CancelSwapIntent>) -> Result<()> {
    let intent = &ctx.accounts.swap_intent;
    let team_wallet = &ctx.accounts.team_wallet;
    let canceller = &ctx.accounts.canceller;
    
    require!(!intent.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!intent.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    
    let is_proposer = canceller.key() == intent.proposer;
    let is_owner = canceller.key() == team_wallet.owner;
    
    require!(
        is_proposer || is_owner,
        TeamWalletError::NotAuthorizedToCancel
    );
    
    msg!("Swap intent cancelled by: {}", canceller.key());
    
    Ok(())
}

/// Cancel expired swap intent (anyone can call)
pub fn cancel_expired_swap_intent(ctx: Context<CancelExpiredSwapIntent>) -> Result<()> {
    let intent = &ctx.accounts.swap_intent;
    let clock = Clock::get()?;
    
    let is_expired = intent.is_expired(clock.unix_timestamp);
    let is_exec_expired = intent.is_execution_expired(clock.unix_timestamp);
    
    require!(
        is_expired || is_exec_expired,
        TeamWalletError::ProposalNotExpired
    );
    
    require!(!intent.executed, TeamWalletError::ProposalAlreadyExecuted);
    
    msg!("Expired swap intent cancelled");
    
    Ok(())
}

/// Close executed swap intent to reclaim rent
pub fn close_swap_intent(ctx: Context<CloseSwapIntent>) -> Result<()> {
    let intent = &ctx.accounts.swap_intent;
    let team_wallet = &ctx.accounts.team_wallet;
    let closer = &ctx.accounts.closer;
    
    require!(intent.executed, TeamWalletError::ProposalNotExecuted);
    
    let is_proposer = closer.key() == intent.proposer;
    let is_owner = closer.key() == team_wallet.owner;
    
    require!(
        is_proposer || is_owner,
        TeamWalletError::NotAuthorizedToCancel
    );
    
    msg!("Executed swap intent closed");
    
    Ok(())
}

#[derive(Accounts)]
pub struct CancelSwapIntent<'info> {
    #[account(
        mut,
        seeds = [
            b"swap_intent",
            team_wallet.key().as_ref(),
            swap_intent.nonce.as_ref(),
        ],
        bump = swap_intent.bump,
        constraint = swap_intent.team_wallet == team_wallet.key(),
        close = proposer
    )]
    pub swap_intent: Account<'info, SwapIntent>,
    
    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    /// CHECK: Receives rent refund
    #[account(
        mut,
        constraint = proposer.key() == swap_intent.proposer @ TeamWalletError::InvalidProposalData
    )]
    pub proposer: AccountInfo<'info>,
    
    pub canceller: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelExpiredSwapIntent<'info> {
    #[account(
        mut,
        seeds = [
            b"swap_intent",
            team_wallet.key().as_ref(),
            swap_intent.nonce.as_ref(),
        ],
        bump = swap_intent.bump,
        constraint = swap_intent.team_wallet == team_wallet.key(),
        close = proposer
    )]
    pub swap_intent: Account<'info, SwapIntent>,
    
    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    /// CHECK: Receives rent refund
    #[account(
        mut,
        constraint = proposer.key() == swap_intent.proposer @ TeamWalletError::InvalidProposalData
    )]
    pub proposer: AccountInfo<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseSwapIntent<'info> {
    #[account(
        mut,
        seeds = [
            b"swap_intent",
            team_wallet.key().as_ref(),
            swap_intent.nonce.as_ref(),
        ],
        bump = swap_intent.bump,
        constraint = swap_intent.team_wallet == team_wallet.key(),
        close = rent_receiver
    )]
    pub swap_intent: Account<'info, SwapIntent>,
    
    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    /// CHECK: Receives rent refund
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,
    
    pub closer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
