use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Deposit amount is insufficient for initialization.")]
    InsufficientDeposit,
    #[msg("Insufficient funds to withdraw.")]
    InsufficientFunds,
    #[msg("Lock period not over.")]
    LockPeriodNotOver,
}
