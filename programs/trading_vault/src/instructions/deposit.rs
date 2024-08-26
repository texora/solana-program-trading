use anchor_lang::prelude::*;
use anchor_spl::token::{self, mint_to, Mint, MintTo, Token, TokenAccount};

use crate::{User, Vault};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
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
    #[account(mut)]
    pub mint: Account<'info, Mint>,
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

    vault.transfer_tokens_from_user(
        ctx.accounts.depositor_pay_token_account.to_account_info(), 
        ctx.accounts.vault_pay_token_account.to_account_info(),
        ctx.accounts.depositor.to_account_info(), 
        ctx.accounts.token_program.to_account_info(),
        params.amount
    )?;

    let bond_amount = (params.amount as f64 / vault.bond_price) as u64;

    let seeds = &["vault_authority".as_bytes(), &[ctx.bumps.vault_authority]];
    let signer = [&seeds[..]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                authority: ctx.accounts.vault_authority.to_account_info(),
                to: ctx.accounts.depositor_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info()
            },
            &signer
        ),
        bond_amount
    );

    vault.tvl += params.amount;
    
    user.deposit_value += params.amount;
    user.bond_amount += bond_amount;
    user.deposit_time = Clock::get()?.unix_timestamp;

    Ok(())
}
