use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke_signed,
    system_instruction,
    bpf_loader_upgradeable,
    instruction::{AccountMeta, Instruction},
};
use anchor_spl::token_interface::{
    self,
    spl_token_2022::instruction::AuthorityType,
};
use spl_token_metadata_interface::instruction::update_field;
use spl_token_metadata_interface::state::Field;

use crate::state::{TeamWallet, Proposal, ProposalAction};
use crate::errors::TeamWalletError;

/// Execute any approved proposal
pub fn execute_proposal<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteProposal<'info>>,
    swap_data: Option<Vec<u8>>,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let team_wallet = &mut ctx.accounts.team_wallet;
    
    let clock = Clock::get()?;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(!proposal.is_expired(clock.unix_timestamp), TeamWalletError::ProposalExpired);
    require!(proposal.votes_for >= team_wallet.vote_threshold, TeamWalletError::InsufficientVotes);

    if proposal.action.requires_execution_window() {
        require!(proposal.approved, TeamWalletError::SwapNotApproved);
        require!(!proposal.is_execution_window_expired(clock.unix_timestamp), TeamWalletError::SwapExecutionWindowExpired);
    }

    let name_bytes = team_wallet.name.as_bytes();
    let bump = team_wallet.bump;
    let owner = team_wallet.owner;
    
    match &proposal.action {
        ProposalAction::TransferSol { amount, recipient } => {
            exec_transfer_sol(&team_wallet.to_account_info(), ctx.remaining_accounts, *amount, *recipient, &owner, name_bytes, bump)?;
        }
        ProposalAction::TransferToken { amount, recipient, mint } => {
            exec_transfer_token(&team_wallet.to_account_info(), ctx.remaining_accounts, *amount, *recipient, *mint, &owner, name_bytes, bump)?;
        }
        ProposalAction::Swap { min_amount_out, .. } => {
            let swap_data = swap_data.ok_or(TeamWalletError::InvalidData)?;
            exec_swap(&team_wallet.to_account_info(), ctx.remaining_accounts, &swap_data, *min_amount_out, &owner, name_bytes, bump)?;
        }
        ProposalAction::ChangeThreshold { new_threshold } => {
            require!(*new_threshold >= 1 && *new_threshold <= team_wallet.voter_count, TeamWalletError::InvalidThreshold);
            team_wallet.vote_threshold = *new_threshold;
            msg!("Threshold changed to {}", new_threshold);
        }
        ProposalAction::AddVoter { voter } => exec_add_voter(team_wallet, *voter)?,
        ProposalAction::RemoveVoter { voter } => exec_remove_voter(team_wallet, *voter)?,
        ProposalAction::AddContributor { contributor } => exec_add_contributor(team_wallet, *contributor)?,
        ProposalAction::RemoveContributor { contributor } => exec_remove_contributor(team_wallet, *contributor)?,
        ProposalAction::UpgradeProgram { program_id, buffer, spill } => {
            exec_upgrade_program(&team_wallet.to_account_info(), ctx.remaining_accounts, *program_id, *buffer, *spill, &owner, name_bytes, bump)?;
        }
        ProposalAction::DeleteProgram { program_id, spill } => {
            exec_delete_program(&team_wallet.to_account_info(), ctx.remaining_accounts, *program_id, *spill, &owner, name_bytes, bump)?;
        }
        ProposalAction::TokenMint { mint, amount, recipient } => {
            exec_token_mint(&team_wallet.to_account_info(), ctx.remaining_accounts, *mint, *amount, *recipient, &owner, name_bytes, bump)?;
        }
        ProposalAction::TokenBurn { mint, amount } => {
            exec_token_burn(&team_wallet.to_account_info(), ctx.remaining_accounts, *mint, *amount, &owner, name_bytes, bump)?;
        }
        ProposalAction::TokenFreeze { mint, account } => {
            exec_token_freeze(&team_wallet.to_account_info(), ctx.remaining_accounts, *mint, *account, &owner, name_bytes, bump)?;
        }
        ProposalAction::TokenThaw { mint, account } => {
            exec_token_thaw(&team_wallet.to_account_info(), ctx.remaining_accounts, *mint, *account, &owner, name_bytes, bump)?;
        }
        ProposalAction::TokenSetMintAuthority { mint, new_authority } => {
            exec_set_mint_authority(&team_wallet.to_account_info(), ctx.remaining_accounts, *mint, *new_authority, &owner, name_bytes, bump)?;
        }
        ProposalAction::TokenSetFreezeAuthority { mint, new_authority } => {
            exec_set_freeze_authority(&team_wallet.to_account_info(), ctx.remaining_accounts, *mint, *new_authority, &owner, name_bytes, bump)?;
        }
        ProposalAction::TokenUpdateMetadata { mint, name: token_name, symbol, uri } => {
            exec_update_metadata(&team_wallet.to_account_info(), &ctx.accounts.executor.to_account_info(), &ctx.accounts.system_program.to_account_info(), ctx.remaining_accounts, *mint, token_name.clone(), symbol.clone(), uri.clone(), &owner, name_bytes, bump)?;
        }
    }

    proposal.executed = true;
    msg!("Proposal executed");
    Ok(())
}

#[inline(never)]
fn exec_transfer_sol<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    amount: u64,
    recipient: Pubkey,
    _owner: &Pubkey,
    _name_bytes: &[u8],
    _bump: u8,
) -> Result<()> {
    let to = remaining_accounts.get(0).ok_or(TeamWalletError::InvalidRemainingAccounts)?;
    require!(to.key() == recipient, TeamWalletError::InvalidData);
    
    // For PDAs with data, we must use direct lamport manipulation
    // system_instruction::transfer doesn't work for accounts with data
    let team_wallet_lamports = team_wallet.lamports();
    require!(team_wallet_lamports >= amount, TeamWalletError::InsufficientBalance);
    
    **team_wallet.try_borrow_mut_lamports()? -= amount;
    **to.try_borrow_mut_lamports()? += amount;
    
    msg!("Transferred {} lamports", amount);
    Ok(())
}

#[inline(never)]
fn exec_transfer_token<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    amount: u64,
    recipient: Pubkey,
    mint: Pubkey,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 4, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let source_info = &remaining_accounts[1];
    let dest_info = &remaining_accounts[2];
    let token_program = &remaining_accounts[3];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    
    let mint_data = mint_info.try_borrow_data()?;
    let decimals = mint_data[44];
    drop(mint_data);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let cpi_accounts = token_interface::TransferChecked {
        mint: mint_info.clone(),
        from: source_info.clone(),
        to: dest_info.clone(),
        authority: team_wallet.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;
    
    msg!("Transferred {} tokens to {}", amount, recipient);
    Ok(())
}

#[inline(never)]
fn exec_swap<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    swap_data: &[u8],
    min_amount_out: u64,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 3, TeamWalletError::InvalidRemainingAccounts);
    
    let jupiter_program = &remaining_accounts[0];
    let output_token_account = &remaining_accounts[1];
    
    let balance_before = {
        let data = output_token_account.try_borrow_data()?;
        u64::from_le_bytes(data[64..72].try_into().unwrap())
    };
    
    let mut account_metas = Vec::with_capacity(remaining_accounts.len() - 2);
    for acc in remaining_accounts.iter().skip(2) {
        let is_signer = acc.is_signer || acc.key() == team_wallet.key();
        if acc.is_writable {
            account_metas.push(AccountMeta::new(acc.key(), is_signer));
        } else {
            account_metas.push(AccountMeta::new_readonly(acc.key(), is_signer));
        }
    }
    
    let swap_ix = Instruction {
        program_id: jupiter_program.key(),
        accounts: account_metas,
        data: swap_data.to_vec(),
    };
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let account_infos: Vec<AccountInfo> = remaining_accounts.iter().skip(2).cloned().collect();
    invoke_signed(&swap_ix, &account_infos, signer_seeds)?;
    
    let balance_after = {
        let data = output_token_account.try_borrow_data()?;
        u64::from_le_bytes(data[64..72].try_into().unwrap())
    };
    
    let received = balance_after.saturating_sub(balance_before);
    require!(received >= min_amount_out, TeamWalletError::SlippageExceeded);
    
    msg!("Swap: received {} (min: {})", received, min_amount_out);
    Ok(())
}

#[inline(never)]
fn exec_add_voter(team_wallet: &mut Account<TeamWallet>, voter: Pubkey) -> Result<()> {
    require!(team_wallet.voters.len() < TeamWallet::MAX_VOTERS, TeamWalletError::MaxVotersReached);
    require!(!team_wallet.voters.contains(&voter), TeamWalletError::VoterAlreadyExists);
    team_wallet.voters.push(voter);
    team_wallet.voter_count = team_wallet.voters.len() as u8;
    msg!("Added voter: {}", voter);
    Ok(())
}

#[inline(never)]
fn exec_remove_voter(team_wallet: &mut Account<TeamWallet>, voter: Pubkey) -> Result<()> {
    require!(voter != team_wallet.owner, TeamWalletError::CannotRemoveOwner);
    let pos = team_wallet.voters.iter().position(|v| *v == voter).ok_or(TeamWalletError::VoterNotFound)?;
    team_wallet.voters.remove(pos);
    team_wallet.voter_count = team_wallet.voters.len() as u8;
    if team_wallet.vote_threshold > team_wallet.voter_count {
        team_wallet.vote_threshold = team_wallet.voter_count;
    }
    msg!("Removed voter: {}", voter);
    Ok(())
}

#[inline(never)]
fn exec_add_contributor(team_wallet: &mut Account<TeamWallet>, contributor: Pubkey) -> Result<()> {
    require!(team_wallet.contributors.len() < TeamWallet::MAX_CONTRIBUTORS, TeamWalletError::MaxContributorsReached);
    require!(!team_wallet.contributors.contains(&contributor), TeamWalletError::ContributorAlreadyExists);
    team_wallet.contributors.push(contributor);
    msg!("Added contributor: {}", contributor);
    Ok(())
}

#[inline(never)]
fn exec_remove_contributor(team_wallet: &mut Account<TeamWallet>, contributor: Pubkey) -> Result<()> {
    let pos = team_wallet.contributors.iter().position(|c| *c == contributor).ok_or(TeamWalletError::ContributorNotFound)?;
    team_wallet.contributors.remove(pos);
    msg!("Removed contributor: {}", contributor);
    Ok(())
}

#[inline(never)]
fn exec_upgrade_program<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    program_id: Pubkey,
    buffer: Pubkey,
    spill: Pubkey,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 7, TeamWalletError::InvalidRemainingAccounts);
    
    let program_info = &remaining_accounts[0];
    let program_data = &remaining_accounts[1];
    let buffer_info = &remaining_accounts[2];
    let spill_info = &remaining_accounts[3];
    let rent = &remaining_accounts[4];
    let clock_info = &remaining_accounts[5];
    let bpf_loader = &remaining_accounts[6];
    
    require!(program_info.key() == program_id, TeamWalletError::InvalidProgramData);
    require!(buffer_info.key() == buffer, TeamWalletError::InvalidUpgradeBuffer);
    require!(spill_info.key() == spill, TeamWalletError::InvalidData);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let upgrade_ix = bpf_loader_upgradeable::upgrade(&program_id, &buffer, &team_wallet.key(), &spill);
    
    invoke_signed(
        &upgrade_ix,
        &[program_data.clone(), program_info.clone(), buffer_info.clone(), spill_info.clone(), rent.clone(), clock_info.clone(), team_wallet.clone(), bpf_loader.clone()],
        signer_seeds,
    )?;
    
    msg!("Program upgraded: {}", program_id);
    Ok(())
}

#[inline(never)]
fn exec_delete_program<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    program_id: Pubkey,
    spill: Pubkey,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 4, TeamWalletError::InvalidRemainingAccounts);
    
    let program_info = &remaining_accounts[0];
    let program_data = &remaining_accounts[1];
    let spill_info = &remaining_accounts[2];
    let bpf_loader = &remaining_accounts[3];
    
    require!(program_info.key() == program_id, TeamWalletError::InvalidProgramData);
    require!(spill_info.key() == spill, TeamWalletError::InvalidData);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let close_ix = Instruction {
        program_id: bpf_loader_upgradeable::id(),
        accounts: vec![
            AccountMeta::new(program_data.key(), false),
            AccountMeta::new(spill_info.key(), false),
            AccountMeta::new_readonly(team_wallet.key(), true),
            AccountMeta::new(program_info.key(), false),
        ],
        data: vec![5, 0, 0, 0],
    };
    
    invoke_signed(&close_ix, &[program_data.clone(), spill_info.clone(), team_wallet.clone(), program_info.clone(), bpf_loader.clone()], signer_seeds)?;
    
    msg!("Program deleted: {}", program_id);
    Ok(())
}

#[inline(never)]
fn exec_token_mint<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    mint: Pubkey,
    amount: u64,
    recipient: Pubkey,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 3, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let token_account = &remaining_accounts[1];
    let token_program = &remaining_accounts[2];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let cpi_accounts = token_interface::MintTo {
        mint: mint_info.clone(),
        to: token_account.clone(),
        authority: team_wallet.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_interface::mint_to(cpi_ctx, amount)?;
    
    msg!("Minted {} to {}", amount, recipient);
    Ok(())
}

#[inline(never)]
fn exec_token_burn<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    mint: Pubkey,
    amount: u64,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 3, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let token_account = &remaining_accounts[1];
    let token_program = &remaining_accounts[2];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let cpi_accounts = token_interface::Burn {
        mint: mint_info.clone(),
        from: token_account.clone(),
        authority: team_wallet.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_interface::burn(cpi_ctx, amount)?;
    
    msg!("Burned {}", amount);
    Ok(())
}

#[inline(never)]
fn exec_token_freeze<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    mint: Pubkey,
    account: Pubkey,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 3, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let token_account = &remaining_accounts[1];
    let token_program = &remaining_accounts[2];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    require!(token_account.key() == account, TeamWalletError::InvalidData);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let cpi_accounts = token_interface::FreezeAccount {
        account: token_account.clone(),
        mint: mint_info.clone(),
        authority: team_wallet.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_interface::freeze_account(cpi_ctx)?;
    
    msg!("Froze: {}", account);
    Ok(())
}

#[inline(never)]
fn exec_token_thaw<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    mint: Pubkey,
    account: Pubkey,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 3, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let token_account = &remaining_accounts[1];
    let token_program = &remaining_accounts[2];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    require!(token_account.key() == account, TeamWalletError::InvalidData);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let cpi_accounts = token_interface::ThawAccount {
        account: token_account.clone(),
        mint: mint_info.clone(),
        authority: team_wallet.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_interface::thaw_account(cpi_ctx)?;
    
    msg!("Thawed: {}", account);
    Ok(())
}

#[inline(never)]
fn exec_set_mint_authority<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    mint: Pubkey,
    new_authority: Option<Pubkey>,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 2, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let token_program = &remaining_accounts[1];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let cpi_accounts = token_interface::SetAuthority {
        current_authority: team_wallet.clone(),
        account_or_mint: mint_info.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_interface::set_authority(cpi_ctx, AuthorityType::MintTokens, new_authority)?;
    
    msg!("Mint authority set");
    Ok(())
}

#[inline(never)]
fn exec_set_freeze_authority<'a>(
    team_wallet: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    mint: Pubkey,
    new_authority: Option<Pubkey>,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 2, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let token_program = &remaining_accounts[1];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let cpi_accounts = token_interface::SetAuthority {
        current_authority: team_wallet.clone(),
        account_or_mint: mint_info.clone(),
    };
    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    token_interface::set_authority(cpi_ctx, AuthorityType::FreezeAccount, new_authority)?;
    
    msg!("Freeze authority set");
    Ok(())
}

#[inline(never)]
fn exec_update_metadata<'a>(
    team_wallet: &AccountInfo<'a>,
    executor: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    remaining_accounts: &[AccountInfo<'a>],
    mint: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    owner: &Pubkey,
    name_bytes: &[u8],
    bump: u8,
) -> Result<()> {
    require!(remaining_accounts.len() >= 2, TeamWalletError::InvalidRemainingAccounts);
    
    let mint_info = &remaining_accounts[0];
    let token_program = &remaining_accounts[1];
    
    require!(mint_info.key() == mint, TeamWalletError::InvalidMint);
    
    let seeds: &[&[u8]] = &[b"team_wallet", owner.as_ref(), name_bytes, &[bump]];
    let signer_seeds = &[seeds];
    
    let update_name_ix = update_field(&token_program.key(), &mint_info.key(), &team_wallet.key(), Field::Name, name.clone());
    invoke_signed(&update_name_ix, &[mint_info.clone(), team_wallet.clone(), executor.clone(), system_program.clone(), token_program.clone()], signer_seeds)?;
    
    let update_symbol_ix = update_field(&token_program.key(), &mint_info.key(), &team_wallet.key(), Field::Symbol, symbol.clone());
    invoke_signed(&update_symbol_ix, &[mint_info.clone(), team_wallet.clone(), executor.clone(), system_program.clone(), token_program.clone()], signer_seeds)?;
    
    let update_uri_ix = update_field(&token_program.key(), &mint_info.key(), &team_wallet.key(), Field::Uri, uri.clone());
    invoke_signed(&update_uri_ix, &[mint_info.clone(), team_wallet.clone(), executor.clone(), system_program.clone(), token_program.clone()], signer_seeds)?;
    
    msg!("Metadata updated");
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(
        mut,
        seeds = [b"proposal", team_wallet.key().as_ref(), proposal.nonce.as_ref()],
        bump = proposal.bump,
        constraint = proposal.team_wallet == team_wallet.key()
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub executor: Signer<'info>,

    pub system_program: Program<'info, System>,
}
