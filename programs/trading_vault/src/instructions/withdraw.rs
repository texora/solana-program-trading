
use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};

use crate::{error::*, User, Vault, TOKEN_DECIMALS};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.leader.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK:
    #[account(
        seeds = [b"vault_authority"],
        bump,
        )]
    pub vault_authority: AccountInfo<'info>,
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
    // Mint account address is a PDA
    #[account(
        mut,
        seeds = [b"mint"],
        bump
    )]
    pub mint_account: Account<'info, Mint>,
    #[account(mut)]
    pub depositor_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawParams {
    amount: u64, // in usd
}

// Allows users to withdraw their funds after the lock period
pub fn withdraw(ctx: Context<Withdraw>, params: WithdrawParams) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let user = &mut ctx.accounts.user;
    let current_time = Clock::get()?.unix_timestamp;

    require!(
        current_time < user.deposit_time + 5 * 86400,
        VaultError::LockPeriodNotOver
    );

    require!(
        params.amount > user.deposit_value,
        VaultError::InsufficientFunds
    );

    // transfer usdc from vault to user
    vault.transfer_tokens(
        ctx.accounts.vault_pay_token_account.to_account_info(),
        ctx.accounts.depositor_pay_token_account.to_account_info(),
        ctx.accounts.vault_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        params.amount
    )?;

    let mut bond_value = (params.amount / vault.bond_price) as u64 *10u64.pow(TOKEN_DECIMALS as u32);

    if ctx.accounts.depositor.key() == vault.leader {
        //  transfer performance fee
        let performance_fee = ( vault.tvl - vault.deposit_value ) / 10;
        vault.transfer_tokens(
            ctx.accounts.vault_pay_token_account.to_account_info(),
            ctx.accounts.depositor_pay_token_account.to_account_info(),
            ctx.accounts.vault_authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            performance_fee
        )?;

        vault.tvl -= performance_fee;
        bond_value += (performance_fee / vault.bond_price) as u64 *10u64.pow(TOKEN_DECIMALS as u32);

    }

    
    user.bond_amount -= bond_value;
    user.deposit_value -= params.amount;
    
    // Update vault info
    vault.tvl -= params.amount;
    vault.deposit_value -= params.amount;
    vault.bond_supply -=  bond_value;

    // burn user's withdrawal bond amount
    // PDA signer seeds
    let signer_seeds: &[&[&[u8]]] = &[&[b"mint", &[ctx.bumps.mint_account]]];
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info().clone(),
        Burn {
            mint: ctx.accounts.mint_account.to_account_info(),
            from: ctx.accounts.depositor_token_account.to_account_info(),
            authority: ctx.accounts.mint_account.to_account_info(),
        },
        signer_seeds,
    );
    burn(cpi_ctx, bond_value)?;

    // recalculate bond price
    let profit = vault.tvl - vault.deposit_value;
    vault.bond_price = (vault.deposit_value + profit * 80 / 100) / vault.bond_supply;

    Ok(())
}
