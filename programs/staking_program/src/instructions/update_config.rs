use crate::{constants::*, states::*};
use anchor_lang::prelude::*;
#[derive(Accounts)]
pub struct ChangePoolSetting<'info> {
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

pub fn handle(
    ctx: Context<ChangePoolSetting>,
    reward_policy_by_class: [u16; CLASS_TYPES],
    lock_day_by_class: [u16; CLASS_TYPES],
    paused: bool,
) -> Result<()> {
    let pool_account = &mut ctx.accounts.pool_account;
    pool_account.paused = paused; // initial status is paused
    pool_account.last_update_time = Clock::get()?.unix_timestamp;
    pool_account.lock_day_by_class = lock_day_by_class;
    pool_account.reward_policy_by_class = reward_policy_by_class;
    Ok(())
}
