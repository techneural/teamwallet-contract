use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use crate::state::{TeamWallet, DeleteProposal};
use crate::errors::TeamWalletError;

pub fn create_delete_proposal(
    ctx: Context<CreateDeleteProposal>,
    program_id: Pubkey,
    spill_account: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.delete_proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    let is_voter = team_wallet.voters.contains(&ctx.accounts.proposer.key());
    let is_contributor = team_wallet.contributors.contains(&ctx.accounts.proposer.key());
    let is_owner = team_wallet.owner == ctx.accounts.proposer.key();

    require!(
        is_voter || is_contributor || is_owner,
        TeamWalletError::NotAVoterOrContributor
    );

    let clock = Clock::get()?;
    let created_at = clock.unix_timestamp;
    let expires_at = created_at + DeleteProposal::DEFAULT_EXPIRY;

    proposal.team_wallet = team_wallet.key();
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.program_id = program_id;
    proposal.spill_account = spill_account;
    proposal.votes_for = 1;
    proposal.voters_voted = vec![ctx.accounts.proposer.key()];
    proposal.votes_against = 0;
    proposal.executed = false;
    proposal.cancelled = false;
    proposal.bump = ctx.bumps.delete_proposal;
    proposal.created_at = created_at;
    proposal.expires_at = expires_at;

    msg!("Delete proposal created, expires at {}", expires_at);
    Ok(())
}

pub fn vote_delete_proposal(
    ctx: Context<VoteDeleteProposal>,
    vote_for: bool,
) -> Result<()> {
    let proposal = &mut ctx.accounts.delete_proposal;
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

pub fn execute_delete_proposal(ctx: Context<ExecuteDeleteProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.delete_proposal;
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

    require!(
        proposal.votes_for >= team_wallet.vote_threshold,
        TeamWalletError::InsufficientVotes
    );

    require!(
        ctx.accounts.program_account.key() == proposal.program_id,
        TeamWalletError::InvalidProgramData
    );

    require!(
        ctx.accounts.spill_account.key() == proposal.spill_account,
        TeamWalletError::InvalidProgramData
    );

    let (expected_program_data, _) = Pubkey::find_program_address(
        &[ctx.accounts.program_account.key().as_ref()],
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

    let close_ix = Instruction {
        program_id: bpf_loader_upgradeable::id(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.program_data.key(), false),
            AccountMeta::new(ctx.accounts.spill_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.team_wallet.key(), true),
            AccountMeta::new(ctx.accounts.program_account.key(), false),
        ],
        data: vec![5, 0, 0, 0], // UpgradeableLoaderInstruction::Close
    };

    invoke_signed(
        &close_ix,
        &[
            ctx.accounts.program_data.to_account_info(),
            ctx.accounts.spill_account.to_account_info(),
            ctx.accounts.team_wallet.to_account_info(),
            ctx.accounts.program_account.to_account_info(),
            ctx.accounts.bpf_loader_upgradeable_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    proposal.executed = true;
    msg!("Program closed successfully");
    Ok(())
}

pub fn close_delete_proposal(ctx: Context<CloseDeleteProposal>) -> Result<()> {
    let proposal = &ctx.accounts.delete_proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);

    let closer_key = ctx.accounts.closer.key();
    require!(
        closer_key == team_wallet.owner ||
        team_wallet.contributors.contains(&closer_key),
        TeamWalletError::NotAVoterOrContributor
    );

    msg!("Delete proposal closed. Rent refunded.");
    Ok(())
}

#[derive(Accounts)]
#[instruction(program_id: Pubkey, spill_account: Pubkey)]
pub struct CreateDeleteProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = DeleteProposal::MAX_SIZE,
        seeds = [b"delete_proposal", team_wallet.key().as_ref(), program_id.as_ref()],
        bump
    )]
    pub delete_proposal: Account<'info, DeleteProposal>,

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
pub struct VoteDeleteProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"delete_proposal",
            team_wallet.key().as_ref(),
            delete_proposal.program_id.as_ref()
        ],
        bump = delete_proposal.bump,
        constraint = delete_proposal.team_wallet == team_wallet.key()
    )]
    pub delete_proposal: Account<'info, DeleteProposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteDeleteProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"delete_proposal",
            team_wallet.key().as_ref(),
            delete_proposal.program_id.as_ref()
        ],
        bump = delete_proposal.bump,
        constraint = delete_proposal.team_wallet == team_wallet.key()
    )]
    pub delete_proposal: Account<'info, DeleteProposal>,

    #[account(
        mut,
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump,
        constraint = delete_proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    /// CHECK: Executor
    #[account(mut)]
    pub executor: Signer<'info>,

    /// CHECK: The program account to close
    #[account(mut)]
    pub program_account: AccountInfo<'info>,

    /// CHECK: ProgramData PDA
    #[account(mut)]
    pub program_data: AccountInfo<'info>,

    /// CHECK: Receives all SOL
    #[account(mut)]
    pub spill_account: AccountInfo<'info>,

    /// CHECK: BPF upgradeable loader
    pub bpf_loader_upgradeable_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CloseDeleteProposal<'info> {
    #[account(
        mut,
        seeds = [
            b"delete_proposal",
            team_wallet.key().as_ref(),
            delete_proposal.program_id.as_ref()
        ],
        bump = delete_proposal.bump,
        constraint = delete_proposal.team_wallet == team_wallet.key(),
        close = spill_account
    )]
    pub delete_proposal: Account<'info, DeleteProposal>,

    #[account(
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub closer: Signer<'info>,

    /// CHECK: Receives rent lamports
    #[account(mut, constraint = spill_account.key() == delete_proposal.spill_account)]
    pub spill_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}
