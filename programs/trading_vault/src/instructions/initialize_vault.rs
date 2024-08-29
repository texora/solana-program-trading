use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token};
use anchor_spl::metadata::{create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
    CreateMetadataAccountsV3, Metadata};

use crate::{constants::TOKEN_DECIMALS, Vault};

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub leader: Signer<'info>,

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
    /// CHECK: Validate address by deriving pda
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
        
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// Initializes the vault with the first depositor as the leader
pub fn initialize_vault(
    ctx: Context<InitializeVault>,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let leader = &mut ctx.accounts.leader;

    vault.vault_authority = ctx.accounts.vault_authority.key();
    vault.vault_authority_bump = ctx.bumps.vault_authority;
    vault.backend_wallet = ctx.accounts.backend_wallet.key();
    vault.strategy_id = "".to_owned();
    vault.bond_price = 1 * 1_000_000;
    vault.deposit_value = 0;
    vault.tvl = 0;
    vault.leader = *leader.to_account_info().key;
    vault.is_trading_paused = false;

    msg!("Creating metadata account");
    
    Ok(())
}
