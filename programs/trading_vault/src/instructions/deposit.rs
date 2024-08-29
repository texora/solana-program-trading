use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::{User, Vault, TOKEN_DECIMALS};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    /// CHECK:
    #[account(mut)]
    pub vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut)]
    pub user: Account<'info, User>,
    // Mint account address is a PDA
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,
    // payment token accounts for deposit
    #[account(mut)]
    pub depositor_pay_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_pay_token_account: Account<'info, TokenAccount>,
    // governance token accounts
    // Create Associated Token Account, if needed
    // This is the account that will hold the minted tokens
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositParams {
    amount: u64, // in usd
}

// Allows any user to deposit into the vault
pub fn deposit(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let user = &mut ctx.accounts.user;

    vault.transfer_tokens_from_user(
        ctx.accounts.depositor_pay_token_account.to_account_info(),
        ctx.accounts.vault_pay_token_account.to_account_info(),
        ctx.accounts.depositor.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        params.amount,
    )?;

    let bond_amount = (params.amount / vault.bond_price) as u64 * 10u64.pow(TOKEN_DECIMALS as u32);

    // PDA signer seeds
    let signer_seeds: &[&[&[u8]]] = &[&[b"mint", &[ctx.bumps.mint_account]]];

    msg!(">>> mint token and assign it to depositor");
    let _ = mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                authority: ctx.accounts.mint_account.to_account_info(),
                to: ctx.accounts.depositor_token_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
            },
            signer_seeds,
        ),
        bond_amount,
    );

    vault.tvl += params.amount;
    vault.deposit_value += params.amount;
    vault.bond_supply += bond_amount;

    user.deposit_value += params.amount;
    user.bond_amount += bond_amount;
    user.deposit_time = Clock::get()?.unix_timestamp;

    // recalculate bond price according to strategy
    let profit = vault.tvl - vault.deposit_value;
    vault.bond_price = (vault.deposit_value + profit * 80 / 100) / vault.bond_supply;

    Ok(())
}
