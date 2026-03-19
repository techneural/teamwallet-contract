use crate::errors::TeamWalletError;
use crate::state::{Proposal, TeamWallet};
use anchor_lang::prelude::*;

pub fn create_proposal_sol(
    ctx: Context<CreateProposalSol>,
    amount: u64,
    recipient: Pubkey,
    _random_pubkey: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    // Validate amount is greater than zero
    require!(amount > 0, TeamWalletError::InvalidAmount);

    let is_voter = team_wallet.voters.contains(&ctx.accounts.proposer.key());
    let is_contributor = team_wallet
        .contributors
        .contains(&ctx.accounts.proposer.key());
    let is_owner = team_wallet.owner == ctx.accounts.proposer.key();

    require!(
        is_voter || is_contributor || is_owner,
        TeamWalletError::NotAVoterOrContributor
    );

    proposal.team_wallet = team_wallet.key();
    proposal.snapshot_voters = team_wallet.voters.clone();
    proposal.snapshot_voters.extend(team_wallet.contributors.clone());

    proposal.proposer = ctx.accounts.proposer.key();
    proposal.amount = amount;
    proposal.recipient = recipient;
    proposal.is_token_transfer = false;
    proposal.mint = None;
    proposal.votes_for = 1;

    let proposer_index = proposal
        .snapshot_voters
        .iter()
        .position(|k| k == &ctx.accounts.proposer.key())
        .unwrap_or(0) as u8;
    proposal.voters_voted = vec![proposer_index];

    proposal.votes_against = 0;
    proposal.executed = false;
    proposal.bump = ctx.bumps.proposal;

    // Initialize swap-related fields to None/false
    proposal.is_swap_proposal = false;
    proposal.input_mint = None;
    proposal.output_mint = None;
    proposal.min_output_amount = None;
    proposal.slippage_bps = None;
    proposal.ready_to_execute = false;

    msg!("SOL proposal created: {} lamports to {}", amount, recipient);

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, recipient: Pubkey, random_pubkey: Pubkey)]
pub struct CreateProposalSol<'info> {
    // FIXED: Changed from init_if_needed to init
    // This prevents reusing an existing proposal PDA
    #[account(
        init,
        payer = proposer,
        space = Proposal::SPACE,
        seeds = [
            b"proposal",
            team_wallet.key().as_ref(),
            random_pubkey.as_ref(),
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

pub fn create_proposal_token(
    ctx: Context<CreateProposalToken>,
    amount: u64,
    recipient: Pubkey,
    mint: Pubkey,
    _random_pubkey: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    // Validate amount is greater than zero
    require!(amount > 0, TeamWalletError::InvalidAmount);

    let is_voter = team_wallet.voters.contains(&ctx.accounts.proposer.key());
    let is_contributor = team_wallet
        .contributors
        .contains(&ctx.accounts.proposer.key());
    let is_owner = team_wallet.owner == ctx.accounts.proposer.key();

    require!(
        is_voter || is_contributor || is_owner,
        TeamWalletError::NotAVoterOrContributor
    );

    proposal.team_wallet = team_wallet.key();

    proposal.snapshot_voters = team_wallet.voters.clone();
    proposal.snapshot_voters.extend(team_wallet.contributors.clone());

    proposal.proposer = ctx.accounts.proposer.key();
    proposal.amount = amount;
    proposal.recipient = recipient;
    proposal.is_token_transfer = true;
    proposal.mint = Some(mint);
    proposal.votes_for = 1;

    let proposer_index = proposal
        .snapshot_voters
        .iter()
        .position(|k| k == &ctx.accounts.proposer.key())
        .unwrap_or(0) as u8;
    proposal.voters_voted = vec![proposer_index];

    proposal.votes_against = 0;
    proposal.executed = false;
    proposal.bump = ctx.bumps.proposal;

    // Initialize swap-related fields to None/false
    proposal.is_swap_proposal = false;
    proposal.input_mint = None;
    proposal.output_mint = None;
    proposal.min_output_amount = None;
    proposal.slippage_bps = None;
    proposal.ready_to_execute = false;

    msg!("Token proposal created: {} tokens to {}", amount, recipient);

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, recipient: Pubkey, mint: Pubkey, random_pubkey: Pubkey)]
pub struct CreateProposalToken<'info> {
    // FIXED: Changed from init_if_needed to init
    // This prevents reusing an existing proposal PDA
    #[account(
        init,
        payer = proposer,
        space = Proposal::SPACE,
        seeds = [
            b"proposal",
            team_wallet.key().as_ref(),
            random_pubkey.as_ref(),
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = proposer,
        associated_token::mint = token_mint,
        associated_token::authority = team_wallet,
        associated_token::token_program = token_program,  
    )]
    pub team_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,  
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
