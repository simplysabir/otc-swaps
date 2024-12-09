use crate::{error::SwapError, events::CancelledSwap};
use crate::state::SwapAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

#[derive(Accounts)]
pub struct CancelSwap<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

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

    /// CHECK: This is the seller's token account, verified in the program logic.
    #[account(mut, associated_token::mint = token_mint, associated_token::authority = seller)]
    pub seller_token_account: Box<Account<'info, TokenAccount>>,

    #[account( address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    #[account( address = anchor_lang::system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handle(ctx: Context<CancelSwap>) -> Result<()> {
    let swap = &mut ctx.accounts.swap;

    require!(swap.is_active, SwapError::SwapNotActive);
    require!(
        swap.seller == ctx.accounts.seller.key(),
        SwapError::UnauthorizedCancellation
    );

    // Return tokens to seller's recipient address
    let transfer_instruction = Transfer {
        from: ctx.accounts.swap_token_account.to_account_info(),
        to: ctx.accounts.seller_token_account.to_account_info(),
        authority: swap.to_account_info(),
    };

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            &[&[b"swap", swap.seller.as_ref(), &[swap.swap_account_bump]]],
        ),
        swap.amount_remaining,
    )?;

    swap.is_active = false;

    emit!(CancelledSwap {
        seller: ctx.accounts.seller.key(),
        amount: swap.total_amount,
        refund: swap.amount_remaining,
        sold: swap.total_amount - swap.amount_remaining,
        token_mint: ctx.accounts.token_mint.key(),
    });

    Ok(())
}
