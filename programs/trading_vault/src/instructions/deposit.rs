use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{User, Vault};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        init_if_needed,
        seeds = [b"user", depositor.key().as_ref()],
        bump,
        payer = depositor,
        space = User::LEN
    )]
    pub user: Account<'info, User>,
    // payment token accounts for deposit
    #[account(mut)]
    pub depositor_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_pay_token_account: Account<'info, TokenAccount>,
    // governance token accounts
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositParams {
    amount: u64,
}

// Allows any user to deposit into the vault
pub fn deposit(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let user = &mut ctx.accounts.user;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.depositor_pay_token_account.to_account_info(),
                to: ctx.accounts.vault_pay_token_account.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        params.amount,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.depositor_token_account.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        params.amount,
    )?;

    vault.tvl += params.amount;
    user.bond_amount += (params.amount as f64 / vault.bond_price) as u64;
    user.deposit_time = Clock::get()?.unix_timestamp;

    Ok(())
}
