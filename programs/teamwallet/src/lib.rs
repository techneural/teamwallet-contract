use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;

use instructions::*;
use state::ProposalAction;

declare_id!("2CeVF4gvaMV6GGF7pSbZgyrP7YtWwALRVAi8CcPP3t72");

#[program]
pub mod teamwallet {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // TEAM WALLET MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════

    /// Initialize a new team wallet
    pub fn initialize_team_wallet(
        ctx: Context<InitializeTeamWallet>,
        name: String,
        vote_threshold: u8,
        voters: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_team_wallet::initialize_team_wallet(
            ctx, name, vote_threshold, voters
        )
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UNIFIED PROPOSAL SYSTEM
    // ═══════════════════════════════════════════════════════════════════════

    /// Create a proposal for any action type
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        action: ProposalAction,
        nonce: Pubkey,
    ) -> Result<()> {
        instructions::create_proposal::create_proposal(ctx, action, nonce)
    }

    /// Vote on any proposal
    pub fn vote_proposal(
        ctx: Context<VoteProposal>,
        vote_for: bool,
    ) -> Result<()> {
        instructions::vote_proposal::vote_proposal(ctx, vote_for)
    }

    /// Execute any approved proposal
    /// 
    /// For swap proposals, pass swap_data from Jupiter.
    /// For other proposals, pass None.
    /// 
    /// Remaining accounts depend on proposal type - see REMAINING_ACCOUNTS.md
    pub fn execute_proposal<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteProposal<'info>>,
        swap_data: Option<Vec<u8>>,
    ) -> Result<()> {
        instructions::execute_proposal::execute_proposal(ctx, swap_data)
    }

    /// Cancel an active proposal (proposer or owner only)
    pub fn cancel_proposal(ctx: Context<CancelProposal>) -> Result<()> {
        instructions::cancel_proposal::cancel_proposal(ctx)
    }

    /// Cancel an expired proposal (anyone can call)
    pub fn cancel_expired_proposal(ctx: Context<CancelExpiredProposal>) -> Result<()> {
        instructions::cancel_proposal::cancel_expired_proposal(ctx)
    }

    /// Close a proposal account (after executed or cancelled)
    pub fn close_proposal(ctx: Context<CloseProposal>) -> Result<()> {
        instructions::cancel_proposal::close_proposal(ctx)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AUTHORITY TRANSFER (Onboarding assets to team wallet)
    // ═══════════════════════════════════════════════════════════════════════

    /// Transfer mint authority to team wallet
    pub fn transfer_mint_authority(ctx: Context<TransferMintAuthority>) -> Result<()> {
        instructions::transfer_authority::transfer_mint_authority(ctx)
    }

    /// Transfer freeze authority to team wallet
    pub fn transfer_freeze_authority(ctx: Context<TransferFreezeAuthority>) -> Result<()> {
        instructions::transfer_authority::transfer_freeze_authority(ctx)
    }

    /// Transfer program upgrade authority to team wallet
    pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {
        instructions::transfer_authority::transfer_program_authority(ctx)
    }
}
