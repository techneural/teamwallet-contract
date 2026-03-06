use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_lang::solana_program::program::{invoke_signed};
use crate::state::{TeamWallet, UpgradeProposal};
use crate::errors::TeamWalletError;

pub fn create_upgrade_proposal(
    ctx: Context<CreateUpgradeProposal>,
    new_buffer: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.upgrade_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    
    let is_voter = team_wallet.voters.contains(&ctx.accounts.proposer.key());
    let is_contributor = team_wallet.contributors.contains(&ctx.accounts.proposer.key());
    let is_owner = team_wallet.owner == ctx.accounts.proposer.key();
    
    require!(
        is_voter || is_contributor || is_owner,
        TeamWalletError::NotAVoterOrContributor
    );
    
    proposal.team_wallet = team_wallet.key();
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.new_buffer = new_buffer;
    proposal.votes_for = 1;
    proposal.voters_voted = vec![ctx.accounts.proposer.key()];
    proposal.votes_against = 0;
    proposal.executed = false;
    proposal.bump = ctx.bumps.upgrade_proposal;
    
    msg!("Upgrade proposal created by: {}", ctx.accounts.proposer.key());
    msg!("New buffer: {}", new_buffer);
    
    Ok(())
}

pub fn execute_upgrade_proposal(ctx: Context<ExecuteUpgradeProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.upgrade_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    
    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    
    let votes_needed = ((team_wallet.voter_count as f64) * (team_wallet.vote_threshold as f64 / 100.0)).ceil() as u8;
    
    require!(
        proposal.votes_for >= votes_needed,
        TeamWalletError::InsufficientVotes
    );

    require!(
        proposal.new_buffer == ctx.accounts.buffer.key(),
        TeamWalletError::InvalidUpgradeBuffer
    );

    let (expected_program_data, _) = Pubkey::find_program_address(
        &[ctx.accounts.program_id.key().as_ref()],
        &bpf_loader_upgradeable::id(),
    );

    require!(
        expected_program_data == ctx.accounts.program_data.key(),
        TeamWalletError::InvalidProgramData
    );
    
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet",
        team_wallet.owner.as_ref(),
        name_bytes,
        &[team_wallet.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    let upgrade_ix = bpf_loader_upgradeable::upgrade(
        &ctx.accounts.program_id.key(),
        &ctx.accounts.buffer.key(),
        &ctx.accounts.spill_account.key(),
        &ctx.accounts.team_wallet.key(),
    );
    
    invoke_signed(
        &upgrade_ix,
        &[
            ctx.accounts.program_data.to_account_info(),
            ctx.accounts.program_id.to_account_info(),
            ctx.accounts.buffer.to_account_info(),
            ctx.accounts.spill_account.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.team_wallet.to_account_info(),
            ctx.accounts.bpf_loader_upgradeable_program.to_account_info(),
        ],
        signer_seeds,
    )?;
    
    proposal.executed = true;
    
    msg!("Program upgraded successfully");
    Ok(())
}

pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;
    
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet",
        team_wallet.owner.as_ref(),
        name_bytes,
        &[team_wallet.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    let set_authority_ix = bpf_loader_upgradeable::set_upgrade_authority(
        &ctx.accounts.program_id.key(),
        &ctx.accounts.current_authority.key(),
        Some(&team_wallet.key()),
    );
    
    invoke_signed(
        &set_authority_ix,
        &[
            ctx.accounts.program_data.to_account_info(),
            ctx.accounts.current_authority.to_account_info(),
            ctx.accounts.team_wallet.to_account_info(),
            ctx.accounts.bpf_loader_upgradeable_program.to_account_info(),
        ],
        signer_seeds,
    )?;
    
    msg!("Program upgrade authority transferred to team wallet");
    Ok(())
}


#[derive(Accounts)]
#[instruction(new_buffer: Pubkey)]
pub struct CreateUpgradeProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + 32 + 32 + 32 + 1 + 1 + 324 + 1 + 1,
        seeds = [b"upgrade_proposal", team_wallet.key().as_ref(),new_buffer.as_ref()],
        bump
    )]
    pub upgrade_proposal: Account<'info, UpgradeProposal>,
    #[account(
    mut,
    seeds = [
        b"team_wallet",
        team_wallet.owner.as_ref(),
        team_wallet.name.as_bytes()
    ],
    bump = team_wallet.bump
)]
    pub team_wallet: Account<'info, TeamWallet>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteUpgradeProposal<'info> {
    #[account(mut)]
    pub upgrade_proposal: Account<'info, UpgradeProposal>,
    
    #[account(
        mut,
        seeds = [
        b"team_wallet",
        team_wallet.owner.as_ref(),
        team_wallet.name.as_bytes()
    ],
    bump = team_wallet.bump,
    constraint = upgrade_proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    pub program_id: AccountInfo<'info>,
    
    #[account(mut)]
    pub program_data: AccountInfo<'info>,
    
    #[account(mut)]
    pub buffer: AccountInfo<'info>,
    
    #[account(mut)]
    pub spill_account: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,

    
    pub bpf_loader_upgradeable_program: AccountInfo<'info>,
    
}


#[derive(Accounts)]
pub struct TransferProgramAuthority<'info> {
    pub team_wallet: Account<'info, TeamWallet>,
    
    #[account(mut)]
    pub program_id: AccountInfo<'info>,
    
    #[account(mut)]
    pub program_data: AccountInfo<'info>,
    
    #[account(mut)]
    pub current_authority: Signer<'info>,
    
    pub bpf_loader_upgradeable_program: AccountInfo<'info>,
}
