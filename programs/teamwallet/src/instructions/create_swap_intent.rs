use anchor_lang::prelude::*;
use crate::state::{TeamWallet, SwapIntent};
use crate::errors::TeamWalletError;

pub fn create_swap_intent(
    ctx: Context<CreateSwapIntent>,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    slippage_bps: u16,
    nonce: Pubkey,
) -> Result<()> {
    let intent = &mut ctx.accounts.swap_intent;
    let team_wallet = &ctx.accounts.team_wallet;
    let proposer = &ctx.accounts.proposer;
    
    let is_voter = team_wallet.voters.contains(&proposer.key());
    let is_contributor = team_wallet.contributors.contains(&proposer.key());
    let is_owner = team_wallet.owner == proposer.key();
    
    require!(
        is_voter || is_contributor || is_owner,
        TeamWalletError::NotAVoterOrContributor
    );
    
    require!(amount_in > 0, TeamWalletError::InvalidAmount);
    require!(min_amount_out > 0, TeamWalletError::InvalidMinOutput);
    require!(input_mint != output_mint, TeamWalletError::SameMintSwap);
    require!(slippage_bps <= 5000, TeamWalletError::SlippageTooHigh);
    
    let clock = Clock::get()?;
    let created_at = clock.unix_timestamp;
    let expires_at = created_at + SwapIntent::DEFAULT_EXPIRY;
    
    let mut snapshot = team_wallet.voters.clone();
    snapshot.extend(team_wallet.contributors.clone());
    
    let proposer_index = snapshot
        .iter()
        .position(|k| k == &proposer.key())
        .unwrap_or(0) as u8;
    
    intent.team_wallet = team_wallet.key();
    intent.proposer = proposer.key();
    intent.input_mint = input_mint;
    intent.output_mint = output_mint;
    intent.amount_in = amount_in;
    intent.min_amount_out = min_amount_out;
    intent.slippage_bps = slippage_bps;
    
    intent.votes_for = 1;
    intent.votes_against = 0;
    intent.voters_voted = vec![proposer_index];
    intent.snapshot_voters = snapshot;
    
    intent.executed = false;
    intent.cancelled = false;
    intent.created_at = created_at;
    intent.expires_at = expires_at;
    intent.approved = false;
    intent.approved_at = 0;
    intent.execution_window_seconds = SwapIntent::DEFAULT_EXECUTION_WINDOW;
    
    intent.bump = ctx.bumps.swap_intent;
    intent.nonce = nonce;
    
    if intent.votes_for >= team_wallet.vote_threshold {
        intent.approved = true;
        intent.approved_at = created_at;
        msg!("Swap intent auto-approved");
    }
    
    msg!("Swap intent created: {} {} -> {}", amount_in, input_mint, output_mint);
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    slippage_bps: u16,
    nonce: Pubkey
)]
pub struct CreateSwapIntent<'info> {
    #[account(
        init,
        payer = proposer,
        space = SwapIntent::SPACE,
        seeds = [
            b"swap_intent",
            team_wallet.key().as_ref(),
            nonce.as_ref(),
        ],
        bump
    )]
    pub swap_intent: Account<'info, SwapIntent>,
    
    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
