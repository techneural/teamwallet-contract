use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{Instruction, AccountMeta},
    program::invoke_signed,
};
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint};
use crate::state::{TeamWallet, SwapIntent};
use crate::errors::TeamWalletError;

/// Execute approved swap via Jupiter
/// 
/// remaining_accounts layout:
/// [0] = Jupiter program
/// [1..n] = All accounts required by Jupiter swap
pub fn execute_swap_intent<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSwapIntent<'info>>,
    swap_data: Vec<u8>,
) -> Result<()> {
    let intent = &mut ctx.accounts.swap_intent;
    let team_wallet = &ctx.accounts.team_wallet;
    
    let clock = Clock::get()?;
    
    require!(!intent.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!intent.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(intent.approved, TeamWalletError::InsufficientVotes);
    require!(
        intent.is_executable(clock.unix_timestamp),
        TeamWalletError::SwapExecutionWindowExpired
    );
    
    require!(
        ctx.accounts.input_token_account.mint == intent.input_mint,
        TeamWalletError::InvalidMint
    );
    require!(
        ctx.accounts.output_token_account.mint == intent.output_mint,
        TeamWalletError::InvalidMint
    );
    require!(
        ctx.accounts.input_token_account.owner == team_wallet.key(),
        TeamWalletError::InvalidTokenAccountOwner
    );
    require!(
        ctx.accounts.input_token_account.amount >= intent.amount_in,
        TeamWalletError::InsufficientBalance
    );
    
    let output_before = ctx.accounts.output_token_account.amount;
    
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet",
        team_wallet.owner.as_ref(),
        name_bytes,
        &[team_wallet.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    require!(
        ctx.remaining_accounts.len() >= 2,
        TeamWalletError::InvalidRemainingAccounts
    );
    
    let jupiter_program = &ctx.remaining_accounts[0];
    
    let mut account_metas: Vec<AccountMeta> = Vec::new();
    
    for account in ctx.remaining_accounts.iter().skip(1) {
        let is_signer = account.key() == team_wallet.key();
        let is_writable = account.is_writable;
        
        if is_signer {
            account_metas.push(AccountMeta::new(account.key(), true));
        } else if is_writable {
            account_metas.push(AccountMeta::new(account.key(), false));
        } else {
            account_metas.push(AccountMeta::new_readonly(account.key(), false));
        }
    }
    
    let swap_ix = Instruction {
        program_id: jupiter_program.key(),
        accounts: account_metas,
        data: swap_data,
    };
    
    let account_infos: Vec<AccountInfo<'info>> = ctx.remaining_accounts
        .iter()
        .cloned()
        .collect();
    
    invoke_signed(&swap_ix, &account_infos, signer_seeds)?;
    
    ctx.accounts.output_token_account.reload()?;
    let output_after = ctx.accounts.output_token_account.amount;
    let output_received = output_after.saturating_sub(output_before);
    
    require!(
        output_received >= intent.min_amount_out,
        TeamWalletError::SlippageExceeded
    );
    
    intent.executed = true;
    
    msg!("Swap executed: {} in, {} out (min {})", 
        intent.amount_in, output_received, intent.min_amount_out);
    
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteSwapIntent<'info> {
    #[account(
        mut,
        seeds = [
            b"swap_intent",
            team_wallet.key().as_ref(),
            swap_intent.nonce.as_ref(),
        ],
        bump = swap_intent.bump,
        constraint = swap_intent.team_wallet == team_wallet.key()
    )]
    pub swap_intent: Account<'info, SwapIntent>,
    
    #[account(
        mut,
        seeds = [b"team_wallet", team_wallet.owner.as_ref(), team_wallet.name.as_bytes()],
        bump = team_wallet.bump
    )]
    pub team_wallet: Account<'info, TeamWallet>,
    
    #[account(
        mut,
        constraint = input_token_account.owner == team_wallet.key() @ TeamWalletError::InvalidTokenAccountOwner
    )]
    pub input_token_account: InterfaceAccount<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = output_token_account.owner == team_wallet.key() @ TeamWalletError::InvalidTokenAccountOwner
    )]
    pub output_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub input_mint: InterfaceAccount<'info, Mint>,
    pub output_mint: InterfaceAccount<'info, Mint>,
    
    pub token_program: Interface<'info, TokenInterface>,
    
    #[account(mut)]
    pub executor: Signer<'info>,
}
