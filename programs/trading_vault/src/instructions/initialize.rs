use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{error::*, User, Vault};

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub leader: Signer<'info>,
    #[account(
        init_if_needed,
        seeds = [b"user", leader.key().as_ref()],
        bump,
        payer = leader,
        space = User::LEN
    )]
    pub user: Account<'info, User>,

    #[account(
        init,
        seeds = [b"vault", leader.key().as_ref()],
        bump,
        payer = leader, 
        space = Vault::LEN
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        seeds = [b"vault_authority"],
        bump,
        )]
    pub vault_authority: AccountInfo<'info>,

    #[account(mut)]
    pub leader_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub leader_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeVaultParams {
    pub strategy_id: String,
    pub initial_deposit: u64
}

// Initializes the vault with the first depositor as the leader
pub fn initialize_vault(
    ctx: Context<InitializeVault>,
    params: InitializeVaultParams
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let leader = &mut ctx.accounts.leader;
    let user = &mut ctx.accounts.user;

    require!(
        params.initial_deposit >= 10 * 1_000_000,
        VaultError::InsufficientDeposit
    ); // 10 USD assuming 6 decimal places

    vault.vault_authority = ctx.accounts.vault_authority.key();
    vault.vault_authority_bump = ctx.bumps.vault_authority;
    vault.strategy_id = params.strategy_id;
    vault.bond_price = 1.0;
    vault.tvl = params.initial_deposit;
    vault.leader = *leader.to_account_info().key;
    vault.is_trading_paused = false;

    user.deposit_value = params.initial_deposit;
    user.bond_amount = params.initial_deposit / 1;
    user.deposit_time = Clock::get()?.unix_timestamp;

    Ok(())
}