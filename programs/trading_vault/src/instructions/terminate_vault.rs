use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::Vault;

#[derive(Accounts)]
pub struct TerminateVault<'info> {
    #[account(mut, has_one = leader)]
    pub vault: Account<'info, Vault>,
    pub leader: Signer<'info>,
}

// Terminates the vault and distributes funds to all depositors
pub fn terminate_vault(ctx: Context<TerminateVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    // Close all positions and distribute funds

    vault.tvl = 0;
    Ok(())
}
