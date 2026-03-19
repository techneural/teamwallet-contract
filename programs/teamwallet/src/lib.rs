#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("jRr8unX8ncDBNiBMfVSUj7D2tUnM7V6199eTiFgJJ7V");

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
        spill_account: Pubkey,
    ) -> Result<()> {
        instructions::create_upgrade_proposal(ctx, new_buffer, spill_account)
    }

    pub fn vote_upgrade_proposal(
        ctx: Context<VoteUpgradeProposal>,
        vote_for: bool,
    ) -> Result<()> {
        instructions::vote_upgrade_proposal(ctx, vote_for)
    }

    pub fn execute_upgrade_proposal(ctx: Context<ExecuteUpgradeProposal>) -> Result<()> {
        instructions::execute_upgrade_proposal(ctx)
    }

    pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {
        instructions::transfer_program_authority(ctx)
    }

    pub fn close_upgrade_proposal(ctx: Context<CloseUpgradeProposal>) -> Result<()> {
        instructions::close_upgrade_proposal(ctx)
    }

    pub fn create_delete_proposal(
        ctx: Context<CreateDeleteProposal>,
        program_id: Pubkey,
        spill_account: Pubkey,
    ) -> Result<()> {
        instructions::create_delete_proposal(ctx, program_id, spill_account)
    }

    pub fn vote_delete_proposal(
        ctx: Context<VoteDeleteProposal>,
        vote_for: bool,
    ) -> Result<()> {
        instructions::vote_delete_proposal(ctx, vote_for)
    }

    pub fn execute_delete_proposal(ctx: Context<ExecuteDeleteProposal>) -> Result<()> {
        instructions::execute_delete_proposal(ctx)
    }

    pub fn close_delete_proposal(ctx: Context<CloseDeleteProposal>) -> Result<()> {
        instructions::close_delete_proposal(ctx)
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

    pub fn set_threshold(ctx: Context<SetThreshold>, new_threshold: u8) -> Result<()> {
        instructions::set_threshold(ctx, new_threshold)
    }

    pub fn create_threshold_proposal(
        ctx: Context<CreateThresholdProposal>,
        new_threshold: u8,
        nonce: Pubkey,
    ) -> Result<()> {
        instructions::create_threshold_proposal(ctx, new_threshold, nonce)
    }

    pub fn vote_threshold_proposal(
        ctx: Context<VoteThresholdProposal>,
        vote_for: bool,
    ) -> Result<()> {
        instructions::vote_threshold_proposal(ctx, vote_for)
    }

    pub fn execute_threshold_proposal(
        ctx: Context<ExecuteThresholdProposal>,
        nonce: Pubkey,
    ) -> Result<()> {
        instructions::execute_threshold_proposal(ctx, nonce)
    }

    // ========== Swap Intent (Jupiter Integration) ==========

    pub fn create_swap_intent(
        ctx: Context<CreateSwapIntent>,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount_in: u64,
        min_amount_out: u64,
        slippage_bps: u16,
        nonce: Pubkey,
    ) -> Result<()> {
        instructions::create_swap_intent(
            ctx, input_mint, output_mint, amount_in, 
            min_amount_out, slippage_bps, nonce
        )
    }

    pub fn vote_swap_intent(ctx: Context<VoteSwapIntent>, vote_for: bool) -> Result<()> {
        instructions::vote_swap_intent(ctx, vote_for)
    }

    pub fn execute_swap_intent<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteSwapIntent<'info>>,
        swap_data: Vec<u8>,
    ) -> Result<()> {
        instructions::execute_swap_intent(ctx, swap_data)
    }

    pub fn cancel_swap_intent(ctx: Context<CancelSwapIntent>) -> Result<()> {
        instructions::cancel_swap_intent(ctx)
    }

    pub fn cancel_expired_swap_intent(ctx: Context<CancelExpiredSwapIntent>) -> Result<()> {
        instructions::cancel_expired_swap_intent(ctx)
    }

    pub fn close_swap_intent(ctx: Context<CloseSwapIntent>) -> Result<()> {
        instructions::close_swap_intent(ctx)
    }
}
