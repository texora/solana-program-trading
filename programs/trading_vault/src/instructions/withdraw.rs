use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{error::*, Vault};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut)]
    pub depositor_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawParams {
    amount: u64
}

// Allows users to withdraw their funds after the lock period
pub fn withdraw(ctx: Context<Withdraw>, params: WithdrawParams) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let depositor = &mut ctx.accounts.depositor;
    let current_time = Clock::get()?.unix_timestamp;

    require!(
        current_time >= depositor.deposit_time + 5 * 86400,
        VaultError::LockPeriodNotOver
    );

    let bond_value = (params.amount as f64 * vault.bond_price) as u64;
    require!(
        bond_value <= depositor.bond_amount,
        VaultError::InsufficientFunds
    );

    // Update vault TVL and depositor's bond amount
    vault.tvl -= bond_value;
    depositor.bond_amount -= params.amount;

    // Calculate performance fee and transfer funds
    let profit = bond_value - depositor.deposit;
    if profit > 0 {
        let performance_fee = profit / 10;
        token::transfer(ctx.accounts.into_transfer_fee_context(), performance_fee)?;
        token::transfer(
            ctx.accounts.into_transfer_context(),
            bond_value - performance_fee,
        )?;
    } else {
        token::transfer(ctx.accounts.into_transfer_context(), bond_value)?;
    }

    Ok(())
}
