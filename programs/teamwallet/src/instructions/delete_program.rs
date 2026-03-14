use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use crate::state::{TeamWallet, DeleteProposal};
use crate::errors::TeamWalletError;

// ─── Create Delete Proposal ───────────────────────────────────────────────────
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

    proposal.team_wallet = team_wallet.key();
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.program_id = program_id;
    proposal.spill_account = spill_account;
    proposal.votes_for = 1;
    proposal.voters_voted = vec![ctx.accounts.proposer.key()];
    proposal.votes_against = 0;
    proposal.executed = false;
    proposal.bump = ctx.bumps.delete_proposal;

    msg!("Delete proposal created for program: {}", program_id);
    msg!("SOL refund destination: {}", spill_account);
    Ok(())
}

// ─── Vote Delete Proposal ─────────────────────────────────────────────────────
pub fn vote_delete_proposal(
    ctx: Context<VoteDeleteProposal>,
    vote_for: bool,
) -> Result<()> {
    let proposal = &mut ctx.accounts.delete_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    let voter_key = ctx.accounts.voter.key();

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);

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

    msg!(
        "Vote: {} voted {} on delete proposal",
        voter_key,
        if vote_for { "FOR" } else { "AGAINST" }
    );
    Ok(())
}

// ─── Execute Delete Proposal ──────────────────────────────────────────────────
// Calls BPF Loader close instruction:
//   close program_data → spill_account (all SOL recovered)
pub fn execute_delete_proposal(ctx: Context<ExecuteDeleteProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.delete_proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);

    // Owner OR any contributor can execute
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

    // Verify program matches proposal
    require!(
        ctx.accounts.program_account.key() == proposal.program_id,
        TeamWalletError::InvalidProgramData
    );

    // Verify spill account matches proposal
    require!(
        ctx.accounts.spill_account.key() == proposal.spill_account,
        TeamWalletError::InvalidProgramData
    );

    // Derive expected program data PDA
    let (expected_program_data, _) = Pubkey::find_program_address(
        &[ctx.accounts.program_account.key().as_ref()],
        &bpf_loader_upgradeable::id(),
    );
    require!(
        expected_program_data == ctx.accounts.program_data.key(),
        TeamWalletError::InvalidProgramData
    );

    // Build PDA signer seeds for team wallet
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet",
        team_wallet.owner.as_ref(),
        name_bytes,
        &[team_wallet.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Build BPF Loader Close instruction manually
    // Discriminant for Close = 5 (u32 LE)
    // Accounts:
    //   0: program_data (writable)      ← what gets closed
    //   1: spill_account (writable)     ← receives all lamports
    //   2: team_wallet PDA (signer)     ← upgrade authority
    //   3: program_account (writable)   ← the program itself
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
    msg!("Program closed successfully. SOL refunded to: {}", ctx.accounts.spill_account.key());
    Ok(())
}

// ─── Close Delete Proposal ───────────────────────────────────────────────────
// Called when cancelling — closes the on-chain PDA so a new proposal can be
// created for the same program later.
pub fn close_delete_proposal(ctx: Context<CloseDeleteProposal>) -> Result<()> {
    let proposal = &ctx.accounts.delete_proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);

    // Owner OR any contributor can close/cancel a delete proposal
    let closer_key = ctx.accounts.closer.key();
    require!(
        closer_key == team_wallet.owner ||
        team_wallet.contributors.contains(&closer_key),
        TeamWalletError::NotAVoterOrContributor
    );

    msg!("Delete proposal closed by: {}. Rent refunded to: {}", closer_key, ctx.accounts.spill_account.key());
    Ok(())
}

// ─── ACCOUNT STRUCTS ──────────────────────────────────────────────────────────

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

    /// CHECK: Must be team wallet owner
    #[account(mut)]
    pub executor: Signer<'info>,

    /// CHECK: The program account to close
    #[account(mut)]
    pub program_account: AccountInfo<'info>,

    /// CHECK: ProgramData PDA — verified in instruction body
    #[account(mut)]
    pub program_data: AccountInfo<'info>,

    /// CHECK: Receives all SOL — verified against proposal.spill_account
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

    /// CHECK: Receives rent lamports — must match proposal.spill_account
    #[account(mut, constraint = spill_account.key() == delete_proposal.spill_account)]
    pub spill_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}