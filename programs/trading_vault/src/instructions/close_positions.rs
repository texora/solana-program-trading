use anchor_lang::prelude::*;

use crate::Vault;

#[derive(Accounts)]
pub struct ClosePositions<'info> {
    #[account(mut, has_one = leader)]
    pub vault: Account<'info, Vault>,
    pub leader: Signer<'info>,
}

// Closes all positions in the vault
pub fn close_positions(ctx: Context<ClosePositions>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    // Logic to close all trading positions via Raydium goes here

    Ok(())
}
