use anchor_lang::prelude::*;
use anchor_spl::token::Transfer;

#[account]
pub struct Vault {
    pub strategy_id: String,
    pub bond_price: u64,
    pub bond_supply: u64,
    pub tvl: u64,
    pub deposit_value: u64,
    pub leader: Pubkey,
    pub is_trading_paused: bool,

    pub backend_wallet: Pubkey,

    pub vault_authority: Pubkey,
    pub vault_authority_bump: u8,
}

impl Vault {
    pub const LEN: usize = std::mem::size_of::<Vault>() + 8;

    pub fn transfer_tokens<'info>(
        &self,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] =
            &[&[b"vault_authority", &[self.vault_authority_bump]]];

        let context = CpiContext::new(
            token_program,
            Transfer {
                from,
                to,
                authority,
            },
        )
        .with_signer(authority_seeds);

        anchor_spl::token::transfer(context, amount)
    }

    pub fn transfer_tokens_from_user<'info>(
        &self,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let context = CpiContext::new(
            token_program,
            Transfer {
                from,
                to,
                authority,
            },
        );
        anchor_spl::token::transfer(context, amount)
    }
}
