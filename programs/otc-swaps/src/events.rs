use anchor_lang::prelude::*;

#[event]
pub struct InitializedSwap {
    pub seller: Pubkey,
    pub amount: u64,
    pub expiry_timestamp: i64,
    pub whitelisted_buyers: [Pubkey; 10],
    pub token_mint: Pubkey,
}

#[event]
pub struct ExecutedSwap {
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
}

#[event]
pub struct CancelledSwap {
    pub seller: Pubkey,
    pub amount: u64,
    pub refund: u64,
    pub sold: u64,
    pub token_mint: Pubkey,
}
