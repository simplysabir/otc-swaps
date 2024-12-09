use anchor_lang::prelude::*;

pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("FDoRMA7k9uXybXW97bBU3979YG1ax4xf3Bf7jaAqYG48");

#[program]
pub mod otc_swaps {
    use super::*;

    pub fn initialize_swap(
        ctx: Context<InitializeSwap>,
        amount: u64,
        expiry_timestamp: i64,
        whitelisted_buyers: [Pubkey; 10],
        recipient: Pubkey,
        amount_in_sol: u64,
    ) -> Result<()> {
        instructions::initialize_swap::handle(
            ctx,
            amount,
            expiry_timestamp,
            whitelisted_buyers,
            recipient,
            amount_in_sol,
        )
    }

    pub fn execute_swap(ctx: Context<ExecuteSwap>, amount_to_buy: u64) -> Result<()> {
        instructions::execute_swap::handle(ctx, amount_to_buy)
    }

    pub fn cancel_swap(ctx: Context<CancelSwap>) -> Result<()> {
        instructions::cancel_swap::handle(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
