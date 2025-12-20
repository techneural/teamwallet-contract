use crate::errors::TeamWalletError;
use crate::state::{Proposal, TeamWallet};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub fn create_proposal(

    ctx: Context<CreateProposal>,

    amount: u64,

    recipient: Pubkey,

    is_token_transfer: bool,

    mint: Option<Pubkey>,

    random_pubkey: Pubkey,          

) -> Result<()> {

    let proposal = &mut ctx.accounts.proposal;

    let team_wallet = &ctx.accounts.team_wallet;
 
    let is_voter = team_wallet.voters.contains(&ctx.accounts.proposer.key());

    let is_contributor = team_wallet

        .contributors

        .contains(&ctx.accounts.proposer.key());

    let is_owner = team_wallet.owner == ctx.accounts.proposer.key();
 
    require!(

        is_voter || is_contributor || is_owner,

        TeamWalletError::NotAVoterOrContributor

    );
 
    if is_token_transfer {

        require!(mint.is_some(), TeamWalletError::MintRequired);

    }
 
    proposal.team_wallet = team_wallet.key();

    proposal.proposer = ctx.accounts.proposer.key();

    proposal.amount = amount;

    proposal.recipient = recipient;

    proposal.is_token_transfer = is_token_transfer;

    proposal.mint = mint;
 
    proposal.votes_for = 1;

    proposal.voters_voted = vec![ctx.accounts.proposer.key()];

    proposal.votes_against = 0;

    proposal.executed = false;

    proposal.bump = ctx.bumps.proposal;
 
    Ok(())

}
 
#[derive(Accounts)]

#[instruction(amount: u64, recipient: Pubkey, is_token_transfer: bool, mint: Option<Pubkey>, random_pubkey: Pubkey)]   // ✅ UPDATED

pub struct CreateProposal<'info> {

    #[account(

        init_if_needed,

        payer = proposer,

        space = 8 + 32 + 32 + 8 + 32 + 1 + 33 + 1 + 1 + 480 + 1 + 1,

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

    )]

    pub team_token_account: Box<Account<'info, TokenAccount>>,
 
    #[account(

        constraint = mint.is_some() @ TeamWalletError::MintRequired

    )]

    pub token_mint: Account<'info, Mint>,
 
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
 
    pub system_program: Program<'info, System>,

}

 