use anchor_lang::prelude::*;
use crate::state::{TeamWallet, Proposal};
use crate::errors::TeamWalletError;

pub fn create_swap_proposal(
    ctx: Context<CreateSwapProposal>,
    amount_in: u64,
    input_mint: Pubkey,
    output_mint: Pubkey,
    min_output_amount: u64,
    slippage_bps: u16,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let wallet = &ctx.accounts.team_wallet;

    let proposer = ctx.accounts.proposer.key();
    require!(
        wallet.owner == proposer
            || wallet.voters.contains(&proposer)
            || wallet.contributors.contains(&proposer),
        TeamWalletError::NotAVoterOrContributor
    );

    require!(amount_in > 0, TeamWalletError::InvalidAmount);
    require!(min_output_amount > 0, TeamWalletError::InvalidMinOutput);
    require!(input_mint != output_mint, TeamWalletError::SameMintSwap);
    require!(slippage_bps <= 1000, TeamWalletError::SlippageTooHigh);

    proposal.team_wallet = wallet.key();
    proposal.proposer = proposer;
    proposal.amount = amount_in;
    proposal.input_mint = Some(input_mint);
    proposal.output_mint = Some(output_mint);
    proposal.min_output_amount = Some(min_output_amount);
    proposal.slippage_bps = Some(slippage_bps);

    proposal.is_token_transfer = false;
    proposal.mint = None;
    proposal.is_swap_proposal = true;

    proposal.votes_for = 1;
    proposal.votes_against = 0;
proposal.snapshot_voters = wallet.voters.clone(); proposal.snapshot_voters.extend(wallet.contributors.clone()); 
     proposal.executed = false;
    proposal.ready_to_execute = false;

    proposal.bump = ctx.bumps.proposal;
    proposal.nonce = wallet.proposal_count;

    Ok(())
}

#[derive(Accounts)]
#[instruction(amount_in: u64, input_mint: Pubkey, output_mint: Pubkey)]


pub struct CreateSwapProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = Proposal::SPACE,
        seeds = [
            b"swap",
            team_wallet.key().as_ref(),
            proposer.key().as_ref(),
            &team_wallet.proposal_count.to_le_bytes(),
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(mut)]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
