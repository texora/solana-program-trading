use anchor_lang::prelude::*;

use crate::Vault;

#[derive(Accounts)]
pub struct PauseTrading<'info> {
    #[account(mut, has_one = leader)]
    pub vault: Account<'info, Vault>,
    pub leader: Signer<'info>,
}

// Pauses trading in the vault
pub fn pause_trading(ctx: Context<PauseTrading>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.is_trading_paused = true;
    Ok(())
}
