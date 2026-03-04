#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("FQigf8ntbnvdzwbJzb9qKH2hmQZhKQwNMKo8iSZ2SwCe");

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
        lookup_table: Pubkey,
    ) -> Result<()> {
        instructions::initialize_team_wallet(ctx, name, vote_threshold, voters, lookup_table)
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

    pub fn create_proposal_sol(
        ctx: Context<CreateProposalSol>,
        amount: u64,
        recipient: Pubkey,
        random_pubkey: Pubkey,
    ) -> Result<()> {
        instructions::create_proposal_sol(ctx, amount, recipient, random_pubkey)
    }

    pub fn create_proposal_token(
        ctx: Context<CreateProposalToken>,
        amount: u64,
        recipient: Pubkey,
        mint: Pubkey,
        random_pubkey: Pubkey,
    ) -> Result<()> {
        instructions::create_proposal_token(ctx, amount, recipient, mint, random_pubkey)
    }

    pub fn vote_proposal(ctx: Context<VoteProposal>, vote_for: bool) -> Result<()> {
        instructions::vote_proposal(ctx, vote_for)
    }

    pub fn vote_token_proposal(ctx: Context<VoteTokenProposal>, vote_for: bool) -> Result<()> {
        instructions::vote_token_proposal(ctx, vote_for)
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

    pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {
        instructions::transfer_program_authority(ctx)
    }

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
        instructions::create_token_proposal(
            ctx, proposal_id, action, amount, recipient,
            metadata, transfer_fee_config, interest_rate,
        )
    }

    pub fn execute_token_proposal(ctx: Context<ExecuteTokenProposal>) -> Result<()> {
        instructions::execute_token_proposal(ctx)
    }

    pub fn transfer_mint_authority(ctx: Context<TransferMintAuthority>) -> Result<()> {
        instructions::transfer_mint_authority(ctx)
    }

    pub fn create_swap_proposal(
        ctx: Context<CreateSwapProposal>,
        amount_in: u64,
        input_mint: Pubkey,
        output_mint: Pubkey,
        min_output_amount: u64,
        slippage_bps: u16,
    ) -> Result<()> {
        instructions::create_swap_proposal(
            ctx, amount_in, input_mint, output_mint, min_output_amount, slippage_bps,
        )
    }

    pub fn vote_swap_proposal(
        ctx: Context<VoteSwapProposal>,
        vote_for: bool,
    ) -> Result<()> {
        instructions::vote_swap_proposal(ctx, vote_for)
    }

    pub fn execute_swap_proposal(
        ctx: Context<ExecuteSwapProposal>,
        route_instructions: Vec<Vec<u8>>,
    ) -> Result<()> {
        instructions::execute_swap_proposal(ctx, route_instructions)
    }

    pub fn set_threshold(ctx: Context<SetThreshold>, new_threshold: u8) -> Result<()> {
        instructions::set_threshold(ctx, new_threshold)
    }

    /// Creates an on-chain ThresholdProposal PDA.
    /// Owner calls this first; then votes are collected off-chain via DB.
    pub fn create_threshold_proposal(
        ctx: Context<CreateThresholdProposal>,
        new_threshold: u8,
        nonce: Pubkey,
    ) -> Result<()> {
        instructions::create_threshold_proposal(ctx, new_threshold, nonce)
    }

    /// Executes the threshold change on-chain after off-chain votes confirm approval.
    /// Closes the proposal account and refunds rent to the owner.
    pub fn execute_threshold_proposal(
        ctx: Context<ExecuteThresholdProposal>,
        nonce: Pubkey,
    ) -> Result<()> {
        instructions::execute_threshold_proposal(ctx, nonce)
    }
}