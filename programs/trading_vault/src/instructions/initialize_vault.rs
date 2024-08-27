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

    #[account(
        init,
        seeds = [b"vault", leader.key().as_ref()],
        bump,
        payer = leader, 
        space = Vault::LEN
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK:
    #[account(
        seeds = [b"vault_authority"],
        bump,
        )]
    pub vault_authority: AccountInfo<'info>,

    // Create mint account
    // Same PDA as address of the account and mint/freeze authority
    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = backend_wallet,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = mint_account.key(),
        mint::freeze_authority = mint_account.key(),

    )]
    pub mint_account: Account<'info, Mint>,
    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
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
    // PDA signer seeds
    let signer_seeds: &[&[&[u8]]] = &[&[b"mint", &[ctx.bumps.mint_account]]];

    // Cross Program Invocation (CPI) signed by PDA
    // Invoking the create_metadata_account_v3 instruction on the token metadata program
    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                mint_authority: ctx.accounts.mint_account.to_account_info(), // PDA is mint authority
                update_authority: ctx.accounts.mint_account.to_account_info(), // PDA is update authority
                payer: ctx.accounts.backend_wallet.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        DataV2 {
            name: "token_name".to_owned(),
            symbol: "token_symbol".to_owned(),
            uri: "token_uri".to_owned(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false, // Is mutable
        true,  // Update authority is signer
        None,  // Collection details
    )?;
    msg!("Token created successfully.");


    Ok(())
}