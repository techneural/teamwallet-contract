#![allow(unexpected_cfgs)]
 
use anchor_lang::prelude::*;
 
declare_id!("5moWiUkB8eVNAG7yqSrAG3pk7FNhADMCvcSYw8XMQnnU");
 
pub mod errors;

pub mod instructions;

pub mod state;
 
use crate::state::{TokenAction, TokenMetadataParams, TransferFeeParams};

use crate::instructions::*;
 
#[program]

pub mod teamwallet {

    use super::*;
 
    pub fn initialize_team_wallet(

        ctx: Context<InitializeTeamWallet>,

        name: String,

        vote_threshold: u8,

        voters: Vec<Pubkey>,

    ) -> Result<()> {

        initialize_team_wallet::initialize_team_wallet(ctx, name, vote_threshold, voters)

    }
 
    pub fn add_voter(ctx: Context<AddVoter>, voter_pubkey: Pubkey) -> Result<()> {

        add_voter::add_voter(ctx, voter_pubkey)

    }
 
    pub fn remove_voter(ctx: Context<RemoveVoter>, voter_pubkey: Pubkey) -> Result<()> {

        remove_voter::remove_voter(ctx, voter_pubkey)

    }
 
    pub fn add_contributor(

        ctx: Context<AddContributor>,

        contributor_pubkey: Pubkey,

    ) -> Result<()> {

        add_contributor::add_contributor(ctx, contributor_pubkey)

    }
 
    pub fn remove_contributor(

        ctx: Context<RemoveContributor>,

        contributor_pubkey: Pubkey,

    ) -> Result<()> {

        remove_contributor::remove_contributor(ctx, contributor_pubkey)

    }
 
    pub fn create_proposal(

    ctx: Context<CreateProposal>,

    amount: u64,

    recipient: Pubkey,

    is_token_transfer: bool,

    mint: Option<Pubkey>,

    random_pubkey: Pubkey,   

) -> Result<()> {

    create_proposal::create_proposal(

        ctx,

        amount,

        recipient,

        is_token_transfer,

        mint,

        random_pubkey,       

    )

}

 
 
    pub fn vote_proposal(ctx: Context<VoteProposal>, vote_for: bool) -> Result<()> {

        vote_proposal::vote_proposal(ctx, vote_for)

    }
 
    pub fn execute_proposal_sol(ctx: Context<ExecuteProposalSol>) -> Result<()> {

        execute_proposal::execute_proposal_sol(ctx)

    }
 
    pub fn execute_proposal_token(ctx: Context<ExecuteProposalToken>) -> Result<()> {

        execute_proposal::execute_proposal_token(ctx)

    }
 
    pub fn create_upgrade_proposal(

        ctx: Context<CreateUpgradeProposal>,

        new_buffer: Pubkey,

    ) -> Result<()> {

        upgrade_program::create_upgrade_proposal(ctx, new_buffer)

    }
 
    pub fn execute_upgrade_proposal(ctx: Context<ExecuteUpgradeProposal>) -> Result<()> {

        upgrade_program::execute_upgrade_proposal(ctx)

    }
 
    pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {

        upgrade_program::transfer_program_authority(ctx)

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
        instructions::create_token_proposal(ctx, proposal_id,action, amount, recipient, metadata, transfer_fee_config, interest_rate)
    }
 
 
    pub fn execute_token_proposal(ctx: Context<ExecuteTokenProposal>) -> Result<()> {

        token_manager::execute_token_proposal(ctx)

    }
 
    pub fn transfer_mint_authority(ctx: Context<TransferMintAuthority>) -> Result<()> {

        token_manager::transfer_mint_authority(ctx)

    }
 
    
pub fn create_swap_proposal(
    ctx: Context<CreateSwapProposal>,
    proposal_id: Pubkey, 
    amount_in: u64,
    min_amount_out: u64,
    input_mint: Pubkey,
    output_mint: Pubkey,
) -> Result<()> {
    instructions::create_swap_proposal(
        ctx,
        proposal_id,
        amount_in,
        min_amount_out,
        input_mint,
        output_mint,
    )
}

 
    /// Execute a swap proposal via Jupiter (or simple transfer on devnet)

    /// 

    /// # Arguments

    /// * `ctx` - The execution context with 'info lifetime for remaining_accounts

    /// * `route_data` - Serialized Jupiter route data (empty for devnet simple transfers)

   pub fn execute_swap_proposal(
    ctx: Context<ExecuteSwapProposal>,
    swap_ix_data: Vec<u8>,
) -> Result<()> {
    instructions::execute_swap_proposal(ctx,swap_ix_data)
}


} 
 