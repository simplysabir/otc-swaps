use anchor_lang::prelude::*;

#[account]
#[derive(Debug, Default, Copy, InitSpace)]
pub struct SwapAccount {
    pub seller: Pubkey,
    pub token_account: Pubkey,
    pub token_mint: Pubkey,
    pub total_amount: u64,
    pub amount_remaining: u64,
    pub expiry_timestamp: i64,
    pub whitelisted_buyers: [Pubkey; 10],
    pub recipient: Pubkey,
    pub is_active: bool,
    pub swap_account_bump: u8,
    pub amount_in_sol: u64,
}
