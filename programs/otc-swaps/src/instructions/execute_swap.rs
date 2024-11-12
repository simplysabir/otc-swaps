use crate::error::SwapError;
use crate::state::SwapAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer, TokenAccount};

#[derive(Accounts)]
pub struct ExecuteSwap<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"swap", swap.seller.as_ref()],
        bump
    )]
    pub swap: Account<'info, SwapAccount>,

    /// CHECK: This is the escrow account holding tokens for the swap, verified in the program logic.
    #[account(mut)]
    pub swap_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: This is the buyer's recipient token account, verified in the program logic.
    #[account(mut)]
    pub buyer_recipient_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
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

pub fn handle(ctx: Context<ExecuteSwap>, recipient_address: Pubkey) -> Result<()> {
    let swap = &mut ctx.accounts.swap;
    let clock = Clock::get()?;

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

    // Validate recipient token account owner
    require!(
        ctx.accounts.buyer_recipient_account.owner.key() == recipient_address,
        SwapError::InvalidRecipientAddress
    );

    verify_spl_token(&swap.token_account, &ctx.accounts.buyer_recipient_account.to_account_info())?;

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
            &[&[b"swap", swap.seller.as_ref(), &[ctx.bumps.swap]]],
        ),
        swap.amount,
    )?;

    // Mark swap as completed
    swap.is_active = false;

    Ok(())
}
