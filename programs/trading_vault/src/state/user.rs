use anchor_lang::prelude::*;

#[account]
pub struct User {
    pub bond_amount: u64,
    pub deposit_value: u64,
    pub deposit_time: i64,
}

impl User {
    pub const LEN: usize = std::mem::size_of::<User>() + 8;
}