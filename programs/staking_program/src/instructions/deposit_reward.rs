use crate::{constants::*, states::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct DepositSwrd<'info> {
    #[account(mut)]
    funder: Signer<'info>,

    #[account(
        mut,
        seeds = [RS_PREFIX.as_bytes()],
        bump,
        constraint = pool_account.is_initialized == true,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = pool_account,
    )]
    reward_vault: Box<Account<'info, TokenAccount>>,

    // funder account
    #[account(mut)]
    funder_account: Account<'info, TokenAccount>,

    #[account(address = pool_account.reward_mint)]
    pub reward_mint: Box<Account<'info, Mint>>,

    // The Token Program
    token_program: Program<'info, Token>,
}

pub fn handle(ctx: Context<DepositSwrd>, amount: u64) -> Result<()> {
    // Transfer reward tokens into the vault.
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        anchor_spl::token::Transfer {
            from: ctx.accounts.funder_account.to_account_info(),
            to: ctx.accounts.reward_vault.to_account_info(),
            authority: ctx.accounts.funder.to_account_info(),
        },
    );

    anchor_spl::token::transfer(cpi_ctx, amount)?;

    Ok(())
}
