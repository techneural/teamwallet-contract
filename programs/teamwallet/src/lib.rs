#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("757e2erDUSUJsRzTXic54JGnddwfY4swFC1t1vrJEq8r");

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;
use crate::state::{TokenAction, TokenMetadataParams, TransferFeeParams};

#[program]
pub mod teamwallet {
    

    use super::*;

    pub fn initialize_team_wallet(
        ctx: Context<InitializeTeamWallet>,
        name: String,
        vote_threshold: u8,
        voters: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_team_wallet(ctx, name, vote_threshold, voters)
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

    pub fn create_upgrade_proposal(
        ctx: Context<CreateUpgradeProposal>,
        new_buffer: Pubkey,
    ) -> Result<()> {
        instructions::create_upgrade_proposal(ctx, new_buffer)
    }

    pub fn execute_upgrade_proposal(ctx: Context<ExecuteUpgradeProposal>) -> Result<()> {
        instructions::execute_upgrade_proposal(ctx)
    }

    // pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {
    //     instructions::transfer_program_authority(ctx)
    // }

    pub fn create_token_proposal(
        ctx: Context<CreateTokenProposal>,
        proposal_id: Pubkey,
        action: TokenAction,
        amount: u64,
        recipient: Option<Pubkey>,
        metadata: Option<TokenMetadataParams>,
        transfer_fee_config: Option<TransferFeeParams>,
        interest_rate: Option<i16>,
    ) -> Result<()> {
        instructions::create_token_proposal(ctx, proposal_id,action, amount, recipient, metadata, transfer_fee_config, interest_rate)
    }

    pub fn execute_token_proposal(ctx: Context<ExecuteTokenProposal>) -> Result<()> {
        instructions::execute_token_proposal(ctx)
    }

    pub fn transfer_mint_authority(ctx: Context<TransferMintAuthority>) -> Result<()> {
        instructions::transfer_mint_authority(ctx)
    }
}