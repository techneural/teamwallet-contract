use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    program::invoke_signed,
};
use anchor_spl::token::{Token, TokenAccount};
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;

pub fn execute_swap_proposal(
    ctx: Context<ExecuteSwapProposal>,
    route_instructions: Vec<Vec<u8>>,
) -> Result<()> {

    let proposal = &mut ctx.accounts.proposal;
    let wallet = &ctx.accounts.team_wallet;

    require!(proposal.is_swap_proposal, TeamWalletError::InvalidProposalType);
    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(proposal.ready_to_execute, TeamWalletError::InsufficientVotes);


    
    let amount_in = proposal.amount;
    let min_out = proposal.min_output_amount.unwrap();

    let name_bytes = wallet.name.as_bytes();  
    let seeds: &[&[u8]] = & [
        b"team_wallet",
        wallet.owner.as_ref(),
        name_bytes,
        &[wallet.bump],
    ];
    let signer_seeds = &[seeds]; 


    let out_balance = ctx.accounts.output_token_account.amount;
    require!(
        out_balance >= min_out,
        TeamWalletError::SlippageExceeded
    );                    
    proposal.executed = true;

    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteSwapProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub input_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub output_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub executor: Signer<'info>,
}
