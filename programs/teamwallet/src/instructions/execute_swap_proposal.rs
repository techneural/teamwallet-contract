use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};

use crate::state::{Proposal, TeamWallet};
use crate::errors::TeamWalletError;

pub fn execute_swap_proposal(
    ctx: Context<ExecuteSwapProposal>,
    swap_ix_data: Vec<u8>,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &ctx.accounts.team_wallet;

  
    require!(
        !proposal.executed,
        TeamWalletError::ProposalAlreadyExecuted
    );

    require!(
        proposal.is_swap_proposal,
        TeamWalletError::InvalidProposalType
    );


    let votes_needed = ((team_wallet.voter_count as f64)
        * (team_wallet.vote_threshold as f64 / 100.0))
        .ceil() as u8;

    require!(
        proposal.votes_for >= votes_needed,
        TeamWalletError::InsufficientVotes
    );

  
    require!(
        ctx.remaining_accounts.len() > 1,
        TeamWalletError::InvalidRemainingAccounts
    );

    let jupiter_program = ctx.remaining_accounts[0].key();

    let metas = ctx
        .remaining_accounts
        .iter()
        .skip(1)
        .map(|acc| AccountMeta {
            pubkey: acc.key(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect::<Vec<_>>();

    let ix = Instruction {
        program_id: jupiter_program,
        accounts: metas,
        data: swap_ix_data,
    };

    let account_infos = ctx
        .remaining_accounts
        .iter()
        .skip(1)
        .map(|acc| acc.to_account_info())
        .collect::<Vec<_>>();

  
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet",
        team_wallet.owner.as_ref(),
        name_bytes,
        &[team_wallet.bump],
    ];
    let signer_seeds = &[&seeds[..]];


    invoke_signed(&ix, &account_infos, signer_seeds)?;


    proposal.executed = true;

    msg!(
        "Swap executed via Jupiter | amount={} min_out={}",
        proposal.amount,
        proposal.min_output_amount.unwrap_or(0)
    );

    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteSwapProposal<'info> {
    #[account(
        mut,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
    pub team_wallet: Account<'info, TeamWallet>,

    /// Executor (any team member)
    pub executor: Signer<'info>,
}
