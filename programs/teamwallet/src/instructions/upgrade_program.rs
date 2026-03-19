use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_lang::solana_program::program::invoke_signed;
use crate::state::{TeamWallet, UpgradeProposal};
use crate::errors::TeamWalletError;

pub fn create_upgrade_proposal(
    ctx: Context<CreateUpgradeProposal>,
    new_buffer: Pubkey,
    spill_account: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.upgrade_proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    let is_voter = team_wallet.voters.contains(&ctx.accounts.proposer.key());
    let is_contributor = team_wallet.contributors.contains(&ctx.accounts.proposer.key());
    let is_owner = team_wallet.owner == ctx.accounts.proposer.key();

    require!(is_voter || is_contributor || is_owner, TeamWalletError::NotAVoterOrContributor);

    let clock = Clock::get()?;
    let created_at = clock.unix_timestamp;
    let expires_at = created_at + UpgradeProposal::DEFAULT_EXPIRY;

    proposal.team_wallet = team_wallet.key();
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.new_buffer = new_buffer;
    proposal.spill_account = spill_account;
    proposal.votes_for = 1;
    proposal.voters_voted = vec![ctx.accounts.proposer.key()];
    proposal.votes_against = 0;
    proposal.executed = false;
    proposal.cancelled = false;
    proposal.bump = ctx.bumps.upgrade_proposal;
    proposal.created_at = created_at;
    proposal.expires_at = expires_at;

    msg!("Upgrade proposal created, expires at {}", expires_at);
    Ok(())
}

pub fn vote_upgrade_proposal(ctx: Context<VoteUpgradeProposal>, vote_for: bool) -> Result<()> {
    let proposal = &mut ctx.accounts.upgrade_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    let voter_key = ctx.accounts.voter.key();
    
    let clock = Clock::get()?;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        !proposal.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalExpired
    );

    let is_voter = team_wallet.voters.contains(&voter_key);
    let is_contributor = team_wallet.contributors.contains(&voter_key);
    let is_owner = team_wallet.owner == voter_key;
    require!(is_voter || is_contributor || is_owner, TeamWalletError::NotAVoter);
    require!(!proposal.voters_voted.contains(&voter_key), TeamWalletError::AlreadyVoted);

    proposal.voters_voted.push(voter_key);
    if vote_for {
        proposal.votes_for = proposal.votes_for.saturating_add(1);
    } else {
        proposal.votes_against = proposal.votes_against.saturating_add(1);
    }

    msg!("Vote: {} voted {}", voter_key, if vote_for { "FOR" } else { "AGAINST" });
    Ok(())
}

pub fn execute_upgrade_proposal(ctx: Context<ExecuteUpgradeProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.upgrade_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    
    let clock = Clock::get()?;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        !proposal.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalExpired
    );

    let executor_key = ctx.accounts.executor.key();
    require!(
        executor_key == team_wallet.owner ||
        team_wallet.contributors.contains(&executor_key),
        TeamWalletError::NotAVoterOrContributor
    );

    require!(proposal.votes_for >= team_wallet.vote_threshold, TeamWalletError::InsufficientVotes);
    require!(proposal.new_buffer == ctx.accounts.buffer.key(), TeamWalletError::InvalidUpgradeBuffer);
    require!(ctx.accounts.spill_account.key() == proposal.spill_account, TeamWalletError::InvalidProgramData);

    let (expected_program_data, _) = Pubkey::find_program_address(
        &[ctx.accounts.program_id.key().as_ref()],
        &bpf_loader_upgradeable::id(),
    );
    require!(expected_program_data == ctx.accounts.program_data.key(), TeamWalletError::InvalidProgramData);

    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[b"team_wallet", team_wallet.owner.as_ref(), name_bytes, &[team_wallet.bump]];
    let signer_seeds = &[&seeds[..]];

    let upgrade_ix = bpf_loader_upgradeable::upgrade(
        &ctx.accounts.program_id.key(),
        &ctx.accounts.buffer.key(),
        &ctx.accounts.team_wallet.key(),
        &ctx.accounts.spill_account.key(),
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

pub fn close_upgrade_proposal(ctx: Context<CloseUpgradeProposal>) -> Result<()> {
    let proposal = &ctx.accounts.upgrade_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    
    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);

    let closer_key = ctx.accounts.proposer.key();
    require!(
        closer_key == team_wallet.owner ||
        team_wallet.contributors.contains(&closer_key),
        TeamWalletError::NotAVoterOrContributor
    );

    msg!("Upgrade proposal closed. Rent refunded.");
    Ok(())
}

pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[b"team_wallet", team_wallet.owner.as_ref(), name_bytes, &[team_wallet.bump]];
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
#[instruction(new_buffer: Pubkey, spill_account: Pubkey)]
pub struct CreateUpgradeProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = UpgradeProposal::MAX_SIZE,
        seeds = [b"upgrade_proposal", team_wallet.key().as_ref(), new_buffer.as_ref()],
        bump
    )]
    pub upgrade_proposal: Account<'info, UpgradeProposal>,

    #[account(
        mut,
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteUpgradeProposal<'info> {
    #[account(
        mut,
        seeds = [b"upgrade_proposal", team_wallet.key().as_ref(), upgrade_proposal.new_buffer.as_ref()],
        bump = upgrade_proposal.bump,
        constraint = upgrade_proposal.team_wallet == team_wallet.key()
    )]
    pub upgrade_proposal: Account<'info, UpgradeProposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteUpgradeProposal<'info> {
    #[account(mut)]
    pub upgrade_proposal: Account<'info, UpgradeProposal>,

    #[account(
        mut,
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump,
        constraint = upgrade_proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    /// CHECK: Executor
    #[account(mut)]
    pub executor: Signer<'info>,

    /// CHECK: Program account being upgraded
    #[account(mut)]
    pub program_id: AccountInfo<'info>,

    /// CHECK: ProgramData PDA
    #[account(mut)]
    pub program_data: AccountInfo<'info>,

    /// CHECK: Upgrade buffer
    #[account(mut)]
    pub buffer: AccountInfo<'info>,

    /// CHECK: Refund address
    #[account(mut)]
    pub spill_account: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,

    /// CHECK: BPF upgradeable loader
    pub bpf_loader_upgradeable_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CloseUpgradeProposal<'info> {
    #[account(
        mut,
        seeds = [b"upgrade_proposal", team_wallet.key().as_ref(), upgrade_proposal.new_buffer.as_ref()],
        bump = upgrade_proposal.bump,
        constraint = upgrade_proposal.team_wallet == team_wallet.key(),
        close = spill_account
    )]
    pub upgrade_proposal: Account<'info, UpgradeProposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    /// CHECK: Receives rent lamports
    #[account(mut, constraint = spill_account.key() == upgrade_proposal.spill_account)]
    pub spill_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferProgramAuthority<'info> {
    pub team_wallet: Account<'info, TeamWallet>,

    /// CHECK: Program whose authority will be transferred
    #[account(mut)]
    pub program_id: AccountInfo<'info>,

    /// CHECK: ProgramData PDA
    #[account(mut)]
    pub program_data: AccountInfo<'info>,

    #[account(mut)]
    pub current_authority: Signer<'info>,

    /// CHECK: BPF upgradeable loader
    pub bpf_loader_upgradeable_program: AccountInfo<'info>,
}
