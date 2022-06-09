use crate::{constants::*, states::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [RS_PREFIX.as_bytes()],
        bump,
        has_one = admin,
        constraint = pool_account.is_initialized == true,
    )]
    pub pool_account: Account<'info, PoolConfig>,
}

pub fn handle(ctx: Context<TransferOwnership>, new_admin: Pubkey) -> Result<()> {
    let pool_account = &mut ctx.accounts.pool_account;
    pool_account.admin = new_admin;
    Ok(())
}
