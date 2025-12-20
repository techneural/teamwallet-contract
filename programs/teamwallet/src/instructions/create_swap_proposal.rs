use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::{TeamWallet, SwapProposal};
use crate::errors::TeamWalletError;


#[derive(Accounts)]
#[instruction(proposal_id: Pubkey)]
pub struct CreateSwapProposal<'info> {
    // Team wallet that owns this swap proposal
    #[account(
        mut,
        seeds = [b"team_wallet", team_wallet.name.as_bytes()],
        bump = team_wallet.bump,
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    // Swap Proposal PDA
    #[account(
        init,
        payer = proposer,
        space = SwapProposal::LEN,
        seeds = [
            b"swap_proposal",
            team_wallet.key().as_ref(),
            proposal_id.as_ref()
        ],
        bump
    )]
    pub swap_proposal: Account<'info, SwapProposal>,

    // Mint we are swapping FROM (input mint)
    pub input_mint: Account<'info, Mint>,

    // Mint we want to receive (output mint)
    pub output_mint: Account<'info, Mint>,

    // Team wallet's token account for input token
    #[account(
        mut,
        constraint = team_wallet_input_ata.owner == team_wallet.key(),
        constraint = team_wallet_input_ata.mint == input_mint.key(),
    )]
    pub team_wallet_input_ata: Account<'info, TokenAccount>,

    // Proposer must pay rent + must be a voter
    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn create_swap_proposal(
    ctx: Context<CreateSwapProposal>,
    proposal_id: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
    input_mint: Pubkey,
    output_mint: Pubkey,
) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;
    let proposer = &ctx.accounts.proposer;
    let proposal = &mut ctx.accounts.swap_proposal;

    //
    // --- VALIDATIONS ---
    //

    // Proposer must be one of the voters
    require!(
        team_wallet.voters.contains(&proposer.key()),
        TeamWalletError::VoterNotFound
    );

    // Input amount must be > 0
    require!(amount_in > 0, TeamWalletError::InvalidAmount);
    require!(min_amount_out > 0, TeamWalletError::InvalidAmount);

    // Ensure the team wallet has enough balance in ATA
    require!(
        ctx.accounts.team_wallet_input_ata.amount >= amount_in,
        TeamWalletError::InsufficientBalance
    );

    //
    // --- INITIALIZE PROPOSAL ---
    //

    proposal.team_wallet = team_wallet.key();
    proposal.proposal_id = proposal_id;
    proposal.proposer = proposer.key();
    proposal.amount_in = amount_in;
    proposal.min_amount_out = min_amount_out;
    proposal.input_mint = input_mint;
    proposal.output_mint = output_mint;

    // Voting
    proposal.votes_for = 1;       // proposer auto votes
    proposal.votes_against = 0;
    proposal.voters = vec![proposer.key()];

    // Metadata
    proposal.executed = false;
    proposal.created_at = Clock::get()?.unix_timestamp;
    proposal.bump = ctx.bumps.swap_proposal;

    msg!(
        "Swap proposal created: {} {} for minimum {} {}",
        amount_in,
        input_mint,
        min_amount_out,
        output_mint
    );

    Ok(())
}
