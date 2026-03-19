use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;

use anchor_spl::token_interface::{
    self,
    spl_token_2022::instruction::AuthorityType,
    Mint,
    TokenAccount,
    TokenInterface,
};

use spl_token_metadata_interface::instruction::update_field;
use spl_token_metadata_interface::state::Field;

use crate::errors::TeamWalletError;
use crate::state::{
    TeamWallet,
    TokenAction,
    TokenMetadataParams,
    TokenProposal,
    TransferFeeParams,
};

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
    let proposal = &mut ctx.accounts.token_proposal;
    let team_wallet = &ctx.accounts.team_wallet;

    let is_voter = team_wallet.voters.contains(&ctx.accounts.proposer.key());
    let is_contributor = team_wallet
        .contributors
        .contains(&ctx.accounts.proposer.key());
    let is_owner = team_wallet.owner == ctx.accounts.proposer.key();

    require!(
        is_voter || is_contributor || is_owner,
        TeamWalletError::NotAVoterOrContributor
    );

    match action {
        TokenAction::Mint => {
            require!(recipient.is_some(), TeamWalletError::RecipientRequired);
        }
        TokenAction::FreezeAccount | TokenAction::ThawAccount => {
            require!(recipient.is_some(), TeamWalletError::RecipientRequired);
        }
        TokenAction::SetMintAuthority => {
            require!(recipient.is_some(), TeamWalletError::RecipientRequired);
        }
        TokenAction::SetFreezeAuthority => {}
        TokenAction::UpdateMetadata => {
            require!(metadata.is_some(), TeamWalletError::MetadataRequired);
        }
        TokenAction::SetTransferFee => {
            require!(
                transfer_fee_config.is_some(),
                TeamWalletError::TransferFeeConfigRequired
            );
        }
        TokenAction::WithdrawTransferFees => {
            require!(recipient.is_some(), TeamWalletError::RecipientRequired);
        }
        TokenAction::EnableConfidentialTransfers
        | TokenAction::DisableConfidentialTransfers => {
            require!(recipient.is_some(), TeamWalletError::RecipientRequired);
        }
        TokenAction::UpdateInterestRate => {
            require!(
                interest_rate.is_some(),
                TeamWalletError::InterestRateRequired
            );
        }
        TokenAction::SetPermanentDelegate => {
            require!(recipient.is_some(), TeamWalletError::RecipientRequired);
        }
        TokenAction::Transfer => {
            require!(recipient.is_some(), TeamWalletError::RecipientRequired);
            require!(amount > 0, TeamWalletError::InvalidAmount);
        }
        _ => {}
    }

    let clock = Clock::get()?;
    let created_at = clock.unix_timestamp;
    let expires_at = created_at + TokenProposal::DEFAULT_EXPIRY;

    proposal.proposal_id = proposal_id;
    proposal.team_wallet = team_wallet.key();

    proposal.snapshot_voters = team_wallet.voters.clone();
    proposal.snapshot_voters.extend(team_wallet.contributors.clone());

    proposal.proposer = ctx.accounts.proposer.key();
    proposal.mint = ctx.accounts.mint.key();
    proposal.action = action;
    proposal.amount = amount;
    proposal.recipient = recipient;
    proposal.metadata = metadata;
    proposal.transfer_fee_config = transfer_fee_config;
    proposal.interest_rate = interest_rate;
    proposal.votes_for = 1;

    let proposer_index = proposal
        .snapshot_voters
        .iter()
        .position(|k| k == &ctx.accounts.proposer.key())
        .unwrap_or(0) as u8;
    proposal.voters_voted = vec![proposer_index];
    proposal.votes_against = 0;
    proposal.executed = false;
    proposal.cancelled = false;
    proposal.bump = ctx.bumps.token_proposal;
    proposal.created_at = created_at;
    proposal.expires_at = expires_at;

    msg!("Token proposal created, expires at {}", expires_at);

    Ok(())
}

pub fn execute_token_proposal(ctx: Context<ExecuteTokenProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.token_proposal;
    let team_wallet = &ctx.accounts.team_wallet;
    
    let clock = Clock::get()?;

    require!(!proposal.executed, TeamWalletError::ProposalAlreadyExecuted);
    require!(!proposal.cancelled, TeamWalletError::ProposalAlreadyCancelled);
    require!(
        !proposal.is_expired(clock.unix_timestamp),
        TeamWalletError::ProposalExpired
    );

    // Use absolute threshold (not float math)
    require!(
        proposal.votes_for >= team_wallet.vote_threshold,
        TeamWalletError::InsufficientVotes
    );

    let name_bytes = team_wallet.name.as_bytes();
    let seeds = &[
        b"team_wallet",
        team_wallet.owner.as_ref(),
        name_bytes,
        &[team_wallet.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    match proposal.action {
        TokenAction::Mint => {
            let cpi_accounts = token_interface::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.as_ref().unwrap().to_account_info(),
                authority: ctx.accounts.team_wallet.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token_interface::mint_to(cpi_ctx, proposal.amount)?;
            msg!("Minted {} tokens", proposal.amount);
        }

        TokenAction::Burn => {
            let cpi_accounts = token_interface::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.token_account.as_ref().unwrap().to_account_info(),
                authority: ctx.accounts.team_wallet.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token_interface::burn(cpi_ctx, proposal.amount)?;
            msg!("Burned {} tokens", proposal.amount);
        }

        TokenAction::FreezeAccount => {
            let cpi_accounts = token_interface::FreezeAccount {
                account: ctx.accounts.token_account.as_ref().unwrap().to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.team_wallet.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token_interface::freeze_account(cpi_ctx)?;
            msg!("Froze token account");
        }

        TokenAction::ThawAccount => {
            let cpi_accounts = token_interface::ThawAccount {
                account: ctx.accounts.token_account.as_ref().unwrap().to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.team_wallet.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token_interface::thaw_account(cpi_ctx)?;
            msg!("Thawed token account");
        }

        TokenAction::SetMintAuthority => {
            let new_authority = proposal.recipient.unwrap();
            let cpi_accounts = token_interface::SetAuthority {
                current_authority: ctx.accounts.team_wallet.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token_interface::set_authority(cpi_ctx, AuthorityType::MintTokens, Some(new_authority))?;
            msg!("Mint authority transferred");
        }

        TokenAction::SetFreezeAuthority => {
            let cpi_accounts = token_interface::SetAuthority {
                current_authority: ctx.accounts.team_wallet.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token_interface::set_authority(cpi_ctx, AuthorityType::FreezeAccount, proposal.recipient)?;
            if proposal.recipient.is_some() {
                msg!("Freeze authority transferred");
            } else {
                msg!("Freeze authority burned (token is now immutable)");
            }
        }

        TokenAction::UpdateMetadata => {
            let meta = proposal
                .metadata
                .as_ref()
                .ok_or(TeamWalletError::MetadataRequired)?;

            let token_program = ctx.accounts.token_program.to_account_info();
            let mint_info = ctx.accounts.mint.to_account_info();
            let team_wallet_info = ctx.accounts.team_wallet.to_account_info();
            let executor_info = ctx.accounts.executor.to_account_info();
            let system_program_info = ctx.accounts.system_program.to_account_info();

            // Update Name
            let update_name_ix = update_field(
                token_program.key,
                mint_info.key,
                team_wallet_info.key,
                Field::Name,
                meta.name.clone(),
            );
            invoke_signed(
                &update_name_ix,
                &[
                    mint_info.clone(),
                    team_wallet_info.clone(),
                    executor_info.clone(),
                    system_program_info.clone(),
                    token_program.clone(),
                ],
                signer_seeds,
            )?;

            // Update Symbol
            let update_symbol_ix = update_field(
                token_program.key,
                mint_info.key,
                team_wallet_info.key,
                Field::Symbol,
                meta.symbol.clone(),
            );
            invoke_signed(
                &update_symbol_ix,
                &[
                    mint_info.clone(),
                    team_wallet_info.clone(),
                    executor_info.clone(),
                    system_program_info.clone(),
                    token_program.clone(),
                ],
                signer_seeds,
            )?;

            // Update URI
            let update_uri_ix = update_field(
                token_program.key,
                mint_info.key,
                team_wallet_info.key,
                Field::Uri,
                meta.uri.clone(),
            );
            invoke_signed(
                &update_uri_ix,
                &[
                    mint_info.clone(),
                    team_wallet_info.clone(),
                    executor_info.clone(),
                    system_program_info.clone(),
                    token_program.clone(),
                ],
                signer_seeds,
            )?;

            msg!(
                "On-chain metadata updated: name={}, symbol={}, uri={}",
                meta.name,
                meta.symbol,
                meta.uri
            );
        }

        TokenAction::Transfer => {
            let source = ctx
                .accounts
                .token_account
                .as_ref()
                .ok_or(TeamWalletError::RecipientRequired)?;

            let destination = ctx
                .accounts
                .destination_token_account
                .as_ref()
                .ok_or(TeamWalletError::RecipientRequired)?;

            let cpi_accounts = token_interface::TransferChecked {
                mint: ctx.accounts.mint.to_account_info(),
                from: source.to_account_info(),
                to: destination.to_account_info(),
                authority: ctx.accounts.team_wallet.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token_interface::transfer_checked(
                cpi_ctx,
                proposal.amount,
                ctx.accounts.mint.decimals,
            )?;
            msg!("Transferred {} tokens to recipient", proposal.amount);
        }

        _ => {}
    }

    proposal.executed = true;
    Ok(())
}

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
    token_interface::set_authority(cpi_ctx, AuthorityType::MintTokens, Some(team_wallet.key()))?;
    msg!("Mint authority transferred to team wallet");
    Ok(())
}

#[derive(Accounts)]
#[instruction(proposal_id: Pubkey)]
pub struct CreateTokenProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = TokenProposal::SPACE,
        seeds = [b"token_proposal", proposal_id.as_ref()],
        bump
    )]
    pub token_proposal: Account<'info, TokenProposal>,

    pub team_wallet: Account<'info, TeamWallet>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteTokenProposal<'info> {
    #[account(mut)]
    pub token_proposal: Account<'info, TokenProposal>,

    #[account(
        mut,
        constraint = token_proposal.team_wallet == team_wallet.key()
    )]
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub token_account: Option<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub destination_token_account: Option<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub executor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferMintAuthority<'info> {
    pub team_wallet: Account<'info, TeamWallet>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    pub current_authority: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}
