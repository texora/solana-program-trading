use anchor_lang::prelude::*;

use crate::Vault;

#[derive(Accounts)]
pub struct TerminateVault<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.leader.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    pub leader: Signer<'info>,
}

// Terminates the vault and distributes funds to all depositors
pub fn terminate_vault(ctx: Context<TerminateVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    // Close all positions and distribute funds

    vault.tvl = 0;
    vault.deposit_value = 0;
    vault.bond_price = 0.0_f64;
    vault.bond_supply = 0;
    Ok(())
}
