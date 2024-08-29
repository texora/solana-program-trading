use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

use crate::{constants::TOKEN_DECIMALS, error::*, User, Vault};

#[derive(Accounts)]
pub struct InitDeposit<'info> {
    #[account(mut)]
    pub leader: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,

    #[account(mut)]
    pub backend_wallet: Signer<'info>,

    #[account(mut)]
    pub vault: Account<'info, Vault>,
    /// CHECK:
    #[account(mut)]
    pub vault_authority: AccountInfo<'info>,

    // Create mint account
    // Same PDA as address of the account and mint/freeze authority
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,

    #[account(mut)]
    pub vault_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub leader_pay_token_account: Account<'info, TokenAccount>,
    // Create Associated Token Account, if needed
    // This is the account that will hold the minted tokens
    #[account(mut)]
    pub leader_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitDepositParams {
    pub strategy_id: String,
    pub initial_deposit: u64,
}

// Initializes the vault with the first depositor as the leader
pub fn init_deposit(
    ctx: Context<InitDeposit>,
    params: InitDepositParams,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let leader = &mut ctx.accounts.leader;
    let user = &mut ctx.accounts.user;

    require!(
        params.initial_deposit >= 10 * 1_000_000,
        VaultError::InsufficientDeposit
    ); // 10 USD assuming 6 decimal places

    vault.strategy_id = params.strategy_id;
    vault.deposit_value = params.initial_deposit;
    vault.tvl = params.initial_deposit;
    vault.leader = *leader.to_account_info().key;

    vault.transfer_tokens_from_user(
        ctx.accounts.leader_pay_token_account.to_account_info(),
        ctx.accounts.vault_pay_token_account.to_account_info(),
        ctx.accounts.vault_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        params.initial_deposit,
    )?;

    let bond_amount =
        params.initial_deposit / 1_000_000 * 10u64.pow(ctx.accounts.mint_account.decimals as u32);

    user.deposit_value = params.initial_deposit;
    user.bond_amount = bond_amount;
    vault.bond_supply = bond_amount;
    user.deposit_time = Clock::get()?.unix_timestamp;

    vault.bond_price = vault.tvl / vault.bond_supply;

    Ok(())
}
