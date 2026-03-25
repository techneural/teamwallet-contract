use anchor_lang::prelude::*;
use crate::state::{TeamWallet, Proposal, ProposalAction};
use crate::errors::TeamWalletError;

/// Create any type of proposal
pub fn create_proposal(
    ctx: Context<CreateProposal>,
    action: ProposalAction,
    nonce: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &mut ctx.accounts.team_wallet;
    let proposer = &ctx.accounts.proposer;

    // Verify proposer is authorized (only owner or contributor can create proposals)
    let is_contributor = team_wallet.contributors.contains(&proposer.key());
    let is_owner = team_wallet.owner == proposer.key();

    require!(
        is_contributor || is_owner,
        TeamWalletError::NotAuthorizedToCreate
    );

    // Validate action-specific requirements
    validate_action(&action)?;

    let clock = Clock::get()?;
    let created_at = clock.unix_timestamp;
    let expires_at = created_at + Proposal::DEFAULT_EXPIRY;

    // Initialize proposal
    proposal.team_wallet = team_wallet.key();
    proposal.proposer = proposer.key();
    proposal.action = action.clone();
    
    // Snapshot includes ALL who can vote: voters + contributors (deduplicated)
    // (owner is already in voters[0])
    let mut snapshot = team_wallet.voters.clone();
    for c in &team_wallet.contributors {
        if !snapshot.contains(c) {
            snapshot.push(*c);
        }
    }
    
    // Snapshot the threshold at creation time for auto-cancel logic
    // This ensures correct behavior even if threshold changes later
    proposal.snapshot_threshold = team_wallet.vote_threshold;
    
    // Find proposer's index for auto-vote
    let proposer_index = snapshot
        .iter()
        .position(|k| k == &proposer.key())
        .unwrap_or(0) as u8;
    
    // Proposer auto-votes
    proposal.votes_for = 1;
    proposal.votes_against = 0;
    proposal.voters_voted = vec![proposer_index];
    proposal.snapshot_voters = snapshot;
    
    proposal.executed = false;
    proposal.cancelled = false;
    proposal.created_at = created_at;
    proposal.expires_at = expires_at;
    
    // For swaps, set execution window
    proposal.execution_window = if action.requires_execution_window() {
        Proposal::DEFAULT_EXECUTION_WINDOW
    } else {
        0
    };
    
    // Check if auto-approved (single signer or threshold met)
    if proposal.votes_for >= team_wallet.vote_threshold {
        proposal.approved = true;
        proposal.approved_at = created_at;
        msg!("Proposal auto-approved");
    } else {
        proposal.approved = false;
        proposal.approved_at = 0;
    }
    
    proposal.bump = ctx.bumps.proposal;
    proposal.nonce = nonce;

    // Increment proposal count
    team_wallet.proposal_count = team_wallet.proposal_count.saturating_add(1);

    msg!("Proposal #{} created, expires at {}", 
        team_wallet.proposal_count, expires_at);

    Ok(())
}

/// Validate action-specific requirements
fn validate_action(action: &ProposalAction) -> Result<()> {
    match action {
        ProposalAction::TransferSol { amount, .. } => {
            require!(*amount > 0, TeamWalletError::InvalidAmount);
        }
        ProposalAction::TransferToken { amount, .. } => {
            require!(*amount > 0, TeamWalletError::InvalidAmount);
        }
        ProposalAction::Swap { 
            input_mint, 
            output_mint, 
            amount_in,
            min_amount_out,
            slippage_bps,
        } => {
            require!(*amount_in > 0, TeamWalletError::InvalidAmount);
            require!(*min_amount_out > 0, TeamWalletError::InvalidAmount);
            require!(input_mint != output_mint, TeamWalletError::SameMintSwap);
            require!(*slippage_bps <= 5000, TeamWalletError::SlippageTooHigh);
        }
        ProposalAction::ChangeThreshold { new_threshold } => {
            require!(*new_threshold >= 1, TeamWalletError::InvalidThreshold);
        }
        ProposalAction::TokenMint { amount, .. } => {
            require!(*amount > 0, TeamWalletError::InvalidAmount);
        }
        ProposalAction::TokenBurn { amount, .. } => {
            require!(*amount > 0, TeamWalletError::InvalidAmount);
        }
        // Other actions don't need validation here
        _ => {}
    }
    Ok(())
}

#[derive(Accounts)]
#[instruction(action: ProposalAction, nonce: Pubkey)]
pub struct CreateProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = Proposal::SPACE,
        seeds = [
            b"proposal",
            team_wallet.key().as_ref(),
            nonce.as_ref(),
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}