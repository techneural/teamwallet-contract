use anchor_lang::prelude::*;
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;

/// Cancel an active proposal (by proposer or owner)
pub fn cancel_proposal(ctx: Context<CancelProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    let canceller = &ctx.accounts.canceller;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);

    // Only proposer or owner can cancel
    let is_proposer = proposal.proposer == canceller.key();
    let is_owner = team_wallet.owner == canceller.key();
    
    require!(
        is_proposer || is_owner,
        TeamWalletError::NotAuthorizedToCancel
    );

    proposal.cancelled = true;
    
    msg!("Proposal cancelled by {}", canceller.key());
    
    Ok(())
}

/// Cancel an expired proposal (anyone can call)
pub fn cancel_expired_proposal(ctx: Context<CancelExpiredProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        proposal.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalNotExpired
    );

    proposal.cancelled = true;
    
    msg!("Expired proposal cancelled");
    
    Ok(())
}

/// Close a proposal account (after executed or cancelled)
pub fn close_proposal(ctx: Context<CloseProposal>) -> Result<()> {
    let proposal = &ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    let closer = &ctx.accounts.closer;

    require!(
        proposal.executed || proposal.cancelled,
        TeamWalletError::ProposalNotExpired // reusing error
    );

    // Only proposer or owner can close
    let is_proposer = proposal.proposer == closer.key();
    let is_owner = team_wallet.owner == closer.key();
    
    require!(
        is_proposer || is_owner,
        TeamWalletError::NotAuthorizedToCancel
    );

    msg!("Proposal account closed, rent returned to {}", ctx.accounts.rent_receiver.key());
    
    Ok(())
}

#[derive(Accounts)]
pub struct CancelProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"proposal",
            team_wallet.key().as_ref(),
            proposal.nonce.as_ref(),
        ],
        bump = proposal.bump,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub canceller: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelExpiredProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"proposal",
            team_wallet.key().as_ref(),
            proposal.nonce.as_ref(),
        ],
        bump = proposal.bump,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"proposal",
            team_wallet.key().as_ref(),
            proposal.nonce.as_ref(),
        ],
        bump = proposal.bump,
        constraint = proposal.team_wallet == team_wallet.key(),
        close = rent_receiver
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub closer: Signer<'info>,

    /// CHECK: Receives the rent
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}
