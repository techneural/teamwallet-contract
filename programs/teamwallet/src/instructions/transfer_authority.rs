use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke_signed,
    bpf_loader_upgradeable,
};
use anchor_spl::token_interface::{
    self,
    spl_token_2022::instruction::AuthorityType,
    Mint,
    TokenInterface,
};

use crate::state::TeamWallet;

/// Transfer mint authority to the team wallet
pub fn transfer_mint_authority(ctx: Context<TransferMintAuthority>) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;

    let cpi_accounts = token_interface::SetAuthority {
        current_authority: ctx.accounts.current_authority.to_account_info(),
        account_or_mint: ctx.accounts.mint.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );
    token_interface::set_authority(
        cpi_ctx, 
        AuthorityType::MintTokens, 
        Some(team_wallet.key())
    )?;
    
    msg!("Mint authority transferred to team wallet: {}", team_wallet.key());
    Ok(())
}

/// Transfer freeze authority to the team wallet
pub fn transfer_freeze_authority(ctx: Context<TransferFreezeAuthority>) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;

    let cpi_accounts = token_interface::SetAuthority {
        current_authority: ctx.accounts.current_authority.to_account_info(),
        account_or_mint: ctx.accounts.mint.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );
    token_interface::set_authority(
        cpi_ctx, 
        AuthorityType::FreezeAccount, 
        Some(team_wallet.key())
    )?;
    
    msg!("Freeze authority transferred to team wallet: {}", team_wallet.key());
    Ok(())
}

/// Transfer program upgrade authority to the team wallet
pub fn transfer_program_authority(ctx: Context<TransferProgramAuthority>) -> Result<()> {
    let team_wallet = &ctx.accounts.team_wallet;
    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet", 
        team_wallet.owner.as_ref(), 
        name_bytes, 
        &[team_wallet.bump]
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

    msg!("Program upgrade authority transferred to team wallet: {}", team_wallet.key());
    Ok(())
}

#[derive(Accounts)]
pub struct TransferMintAuthority<'info> {
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    pub current_authority: Signer<'info>,
    
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct TransferFreezeAuthority<'info> {
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    pub current_authority: Signer<'info>,
    
    pub token_program: Interface<'info, TokenInterface>,
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
