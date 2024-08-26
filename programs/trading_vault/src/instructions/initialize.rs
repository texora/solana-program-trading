use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use anchor_spl::metadata::{create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
    CreateMetadataAccountsV3, Metadata};

use crate::{error::*, constants::TOKEN_DECIMALS, User, Vault};

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

    #[account(mut)]
    pub backend_wallet: AccountInfo<'info>,

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
    
    #[account(mut)]
    pub leader_pay_token_account: Account<'info, TokenAccount>,
    // Create Associated Token Account, if needed
    // This is the account that will hold the minted tokens
    #[account(
        init_if_needed,
        payer = leader,
        associated_token::mint = mint_account,
        associated_token::authority = leader,
    )]
    pub leader_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
    vault.backend_wallet = ctx.accounts.backend_wallet.key();
    vault.strategy_id = params.strategy_id;
    vault.bond_price = 1.0;
    vault.tvl = params.initial_deposit;
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

    let bond_amount = params.initial_deposit / 1;
    user.deposit_value = params.initial_deposit;
    user.bond_amount = bond_amount;
    user.deposit_time = Clock::get()?.unix_timestamp;

    // Invoke the mint_to instruction on the token program
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint_account.to_account_info(),
                to: ctx.accounts.leader_token_account.to_account_info(),
                authority: ctx.accounts.mint_account.to_account_info(), // PDA mint authority, required as signer
            },
        )
        .with_signer(signer_seeds), // using PDA to sign
        bond_amount * 10u64.pow(ctx.accounts.mint_account.decimals as u32), // Mint tokens, adjust for decimals
    )?;
    msg!("Token minted successfully.");


    Ok(())
}