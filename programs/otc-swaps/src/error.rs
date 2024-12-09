use anchor_lang::prelude::*;

#[error_code]
pub enum SwapError {
    #[msg("Swap has expired")]
    SwapExpired,

    #[msg("Swap is not active")]
    SwapNotActive,

    #[msg("Buyer is not whitelisted")]
    BuyerNotWhitelisted,

    #[msg("Invalid expiry time")]
    InvalidExpiryTime,

    #[msg("Unauthorized cancellation attempt")]
    UnauthorizedCancellation,

    #[msg("Amount must be greater than 0")]
    InvalidAmount,

    #[msg("Whitelist cannot be empty")]
    EmptyWhitelist,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Token account is frozen")]
    TokenAccountFrozen,

    #[msg("Inavlid Recipient Address")]
    InvalidRecipientAddress,

    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Invalid amount to buy")]
    InvalidAmountToBuy,
}
