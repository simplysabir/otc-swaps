use crate::error::SwapError;
use crate::events::ExecutedSwap;
use crate::state::SwapAccount;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct ExecuteSwap<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: This is the seller's token account, verified in the program logic.
    #[account(mut, address = swap.seller)]
    pub seller: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"swap", seller.key().as_ref(), token_mint.key().as_ref()],
        bump = swap.swap_account_bump
    )]
    pub swap: Account<'info, SwapAccount>,

    /// CHECK: This is the token mint account, verified in the program logic.
    #[account(mut, address = swap.token_mint)]
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: This is the escrow account holding tokens for the swap, verified in the program logic.
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = swap)]
    pub swap_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: This is the buyer's recipient token account, verified in the program logic.
    #[account(mut, token::mint  = token_mint, token::authority = buyer)]
    pub buyer_recipient_account: Box<Account<'info, TokenAccount>>,

    #[account( address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account( address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    #[account( address = anchor_lang::system_program::ID)]
    pub system_program: Program<'info, System>,
}

// Helper function to verify SPL token
fn verify_spl_token(token_mint: &Pubkey, token_account: &AccountInfo) -> Result<()> {
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

pub fn handle(ctx: Context<ExecuteSwap>, amount_to_buy: u64) -> Result<()> {
    let swap = &mut ctx.accounts.swap;
    let clock = Clock::get()?;

    // Check if the buyer is trying to buy more than the swap amount
    require!(amount_to_buy <= swap.amount_remaining, SwapError::InvalidAmountToBuy);

    // Validate swap state
    require!(swap.is_active, SwapError::SwapNotActive);
    require!(
        clock.unix_timestamp <= swap.expiry_timestamp,
        SwapError::SwapExpired
    );

    // Verify buyer is whitelisted
    require!(
        swap.whitelisted_buyers.contains(&ctx.accounts.buyer.key()),
        SwapError::BuyerNotWhitelisted
    );

    verify_spl_token(
        &swap.token_account,
        &ctx.accounts.buyer_recipient_account.to_account_info(),
    )?;

    // Calculate price per token
    let price_per_token = swap.amount_in_sol as f64 / swap.total_amount as f64;
    let sol_required = (amount_to_buy as f64 * price_per_token) as u64;

    // Ensure buyer has sufficient SOL
    require!(
        ctx.accounts.buyer.lamports() >= sol_required,
        SwapError::InsufficientBalance
    );

    // Transfer SOL from buyer to seller
    **ctx.accounts.buyer.lamports.borrow_mut() -= sol_required;
    **ctx.accounts.seller.lamports.borrow_mut() += sol_required;

    // Transfer tokens from escrow to buyer's recipient
    let transfer_instruction = Transfer {
        from: ctx.accounts.swap_token_account.to_account_info(),
        to: ctx.accounts.buyer_recipient_account.to_account_info(),
        authority: swap.to_account_info(),
    };

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            &[&[b"swap", swap.seller.as_ref(), &[swap.swap_account_bump]]],
        ),
        amount_to_buy,
    )?;

    // Reduce the swap's token amount
    swap.amount_remaining -= amount_to_buy;

    // Deactivate the swap if no tokens are left
    if swap.amount_remaining == 0 {
        swap.is_active = false;
    }

    emit!(ExecutedSwap {
        seller: swap.seller,
        buyer: ctx.accounts.buyer.key(),
        amount: amount_to_buy,
        token_mint: ctx.accounts.token_mint.key(),
    });

    Ok(())
}
