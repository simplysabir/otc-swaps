use anchor_lang::prelude::*;

#[account]
pub struct SwapAccount {
    pub seller: Pubkey,
    pub token_account: Pubkey,
    pub amount: u64,
    pub expiry_timestamp: i64,
    pub whitelisted_buyers: [Pubkey;10],
    pub recipient: Pubkey,
    pub is_active: bool,
}

impl SwapAccount {
    pub const MAX_WHITELIST_SIZE: usize = 10;
    pub const LEN: usize = 8 + // discriminator
        32 + // seller
        32 + // token_account
        8 + // amount
        8 + // expiry_timestamp
        (32 * Self::MAX_WHITELIST_SIZE) + // whitelisted_buyers (max 10 buyers)
        32 + // recipient
        1; // is_active
}
