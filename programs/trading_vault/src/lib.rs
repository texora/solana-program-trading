pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4aeW1288H4t5oSmUhmrxmVfvuhFgYrtPSj6BGwCC4djv");

#[program]
pub mod trading_vault {
    use super::*;

    pub fn initialize(ctx: Context<InitializeVault>, params: InitializeVaultParams) -> Result<()> {
        initialize::initialize_vault(ctx, params)
    }

    pub fn deposit(ctx: Context<Deposit>, params: DepositParams) -> Result<()> {
        deposit::deposit(ctx, params)
    }

    pub fn withdraw(ctx: Context<Withdraw>, params: WithdrawParams) -> Result<()> {
        withdraw::withdraw(ctx, params)
    }

    pub fn pause_trading(ctx: Context<PauseTrading>) -> Result<()> {
        pause_trading::pause_trading(ctx)
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        close_position::close_position(ctx)
    }

    pub fn terminate_vault(ctx: Context<TerminateVault>) -> Result<()> {
        terminate_vault::terminate_vault(ctx)
    }
}
