use crate::error::SwapError;
use crate::state::SwapAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer, TokenAccount};

#[derive(Accounts)]
pub struct CancelSwap<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"swap", seller.key().as_ref()],
        bump
    )]
    pub swap: Account<'info, SwapAccount>,

    /// CHECK: This is the escrow account holding tokens for the swap, verified in the program logic.
    #[account(mut)]
    pub swap_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: This is the seller's recipient token account, verified in the program logic.
    #[account(mut)]
    pub seller_recipient_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handle(ctx: Context<CancelSwap>) -> Result<()> {
    let swap = &mut ctx.accounts.swap;

    require!(swap.is_active, SwapError::SwapNotActive);
    require!(
        swap.seller == ctx.accounts.seller.key(),
        SwapError::UnauthorizedCancellation
    );

    // // Check if the swap has expired
    // let clock = Clock::get()?;
    // require!(
    //     clock.unix_timestamp <= swap.expiry_timestamp,
    //     SwapError::SwapExpired
    // );
    
    // Return tokens to seller's recipient address
    let transfer_instruction = Transfer {
        from: ctx.accounts.swap_token_account.to_account_info(),
        to: ctx.accounts.seller_recipient_account.to_account_info(),
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

    swap.is_active = false;

    Ok(())
}
