use anchor_lang::prelude::*;

declare_id!("4CTGUdAt49S9CNcUWyHyCXyYZNvg2QiLxdMrRHDUcrtj");

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;
use errors::*;

#[program]
pub mod teamwallet {
    use super::*;

    pub fn initialize_team_wallet(
        ctx: Context<InitializeTeamWallet>,
        name: String,
        vote_threshold: u8,
    ) -> Result<()> {
        instructions::initialize_team_wallet(ctx, name, vote_threshold)
    }

    pub fn add_voter(ctx: Context<AddVoter>, voter_pubkey: Pubkey) -> Result<()> {
        instructions::add_voter(ctx, voter_pubkey)
    }

    pub fn remove_voter(ctx: Context<RemoveVoter>, voter_pubkey: Pubkey) -> Result<()> {
        instructions::remove_voter(ctx, voter_pubkey)
    }

    pub fn add_contributor(ctx: Context<AddContributor>, contributor_pubkey: Pubkey) -> Result<()> {
        instructions::add_contributor(ctx, contributor_pubkey)
    }

    pub fn remove_contributor(ctx: Context<RemoveContributor>, contributor_pubkey: Pubkey) -> Result<()> {
        instructions::remove_contributor(ctx, contributor_pubkey)
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        amount: u64,
        recipient: Pubkey,
        is_token_transfer: bool,
        mint: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_proposal(ctx, amount, recipient, is_token_transfer, mint)
    }

    pub fn vote_proposal(ctx: Context<VoteProposal>, vote_for: bool) -> Result<()> {
        instructions::vote_proposal(ctx, vote_for)
    }

    pub fn execute_proposal_sol(ctx: Context<ExecuteProposalSol>) -> Result<()> {
        instructions::execute_proposal_sol(ctx)
    }

    pub fn execute_proposal_token(ctx: Context<ExecuteProposalToken>) -> Result<()> {
        instructions::execute_proposal_token(ctx)
    }
}