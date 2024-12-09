use crate::error::SwapError;
use crate::events::InitializedSwap;
use crate::state::SwapAccount;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct InitializeSwap<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        init,
        payer = seller,
        space = SwapAccount::INIT_SPACE,
        seeds = [b"swap", seller.key().as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub swap: Account<'info, SwapAccount>,

    /// CHECK: This is the seller's token account, verified in the program logic.
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = seller)]
    pub seller_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: This is the token mint account, verified in the program logic.
    #[account(mut)]
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: This is the escrow account holding tokens for the swap, verified in the program logic.
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = swap)]
    pub swap_token_account: Box<Account<'info, TokenAccount>>,

    #[account( address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account( address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    #[account( address = anchor_lang::system_program::ID)]
    pub system_program: Program<'info, System>,
}

// Helper function to verify SPL token
fn verify_spl_token(token_mint: &AccountInfo, token_account: &AccountInfo) -> Result<()> {
    // Deserialize the token account
    let token_account_data: TokenAccount =
        TokenAccount::try_deserialize(&mut &**token_account.data.borrow())?;

    // Verify the token account's mint matches the provided mint
    require!(
        token_account_data.mint.key() == token_mint.key(),
        SwapError::InvalidTokenMint
    );

    // Verify token account is not frozen
    require!(
        !token_account_data.is_frozen(),
        SwapError::TokenAccountFrozen
    );
    Ok(())
}

pub fn handle(
    ctx: Context<InitializeSwap>,
    amount: u64,
    expiry_timestamp: i64,
    whitelisted_buyers: [Pubkey; 10],
    recipient: Pubkey,
    amount_in_sol: u64,
) -> Result<()> {
    let swap = &mut ctx.accounts.swap;
    let clock = Clock::get()?;

    // Verify this is a valid SPL token
    verify_spl_token(
        &ctx.accounts.token_mint.to_account_info(),
        &ctx.accounts.seller_token_account.to_account_info(),
    )?;

    // Validate inputs
    require!(amount > 0, SwapError::InvalidAmount);
    require!(!whitelisted_buyers.is_empty(), SwapError::EmptyWhitelist);
    require!(
        expiry_timestamp > clock.unix_timestamp
            && expiry_timestamp <= clock.unix_timestamp + 48 * 3600,
        SwapError::InvalidExpiryTime
    );

    let token_account_data = &ctx.accounts.seller_token_account;
    require!(
        token_account_data.amount >= amount,
        SwapError::InsufficientBalance
    );

    // Initialize swap state
    swap.seller = ctx.accounts.seller.key();
    swap.token_account = ctx.accounts.seller_token_account.key();
    swap.token_mint = ctx.accounts.token_mint.key();
    swap.amount_remaining = amount;
    swap.total_amount = amount;
    swap.expiry_timestamp = expiry_timestamp;
    swap.whitelisted_buyers = whitelisted_buyers;
    swap.recipient = recipient;
    swap.is_active = true;
    swap.amount_in_sol = amount_in_sol;

    // Transfer tokens to swap escrow
    let transfer_instruction = Transfer {
        from: ctx.accounts.seller_token_account.to_account_info(),
        to: ctx.accounts.swap_token_account.to_account_info(),
        authority: ctx.accounts.seller.to_account_info(),
    };

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        ),
        amount,
    )?;

    emit!(InitializedSwap {
        seller: ctx.accounts.seller.key(),
        amount,
        expiry_timestamp,
        whitelisted_buyers,
        token_mint: ctx.accounts.token_mint.key(),
    });

    Ok(())
}
