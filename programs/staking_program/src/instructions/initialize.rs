use crate::{constants::*, error::*, states::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitializeStakingPool<'info> {
    // The pool owner
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        seeds = [RS_PREFIX.as_bytes()],
        bump,
        payer = admin,
        space = 8 + std::mem::size_of::<PoolConfig>(),
    )]
    pub pool_account: Account<'info, PoolConfig>,

    // reward mint
    pub reward_mint: Account<'info, Mint>,

    // reward vault that holds the reward mint for distribution
    #[account(
        init,
        token::mint = reward_mint,
        token::authority = pool_account,
        seeds = [ RS_VAULT_SEED.as_bytes(), reward_mint.key().as_ref() ],
        bump,
        payer = admin,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    // The rent sysvar
    pub rent: Sysvar<'info, Rent>,
    // system program
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,

    // token program
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

impl<'info> InitializeStakingPool<'info> {
    pub fn validate(&self) -> Result<()> {
        if self.pool_account.is_initialized == true {
            require!(
                self.pool_account.admin.eq(&self.admin.key()),
                StakingError::NotAllowedAuthority
            )
        }
        Ok(())
    }
}

/**
 * Initialize Staking program for the first time to init staking pool config with some data for validation.
 */
#[access_control(ctx.accounts.validate())]
pub fn initialize_staking_pool(
    ctx: Context<InitializeStakingPool>,
    reward_policy_by_class: [u16; CLASS_TYPES],
    lock_day_by_class: [u16; CLASS_TYPES],
) -> Result<()> {
    msg!("initializing");

    let pool_account = &mut ctx.accounts.pool_account;

    pool_account.is_initialized = true;
    pool_account.admin = *ctx.accounts.admin.key;
    pool_account.paused = false; // initial status is paused
    pool_account.reward_mint = *ctx.accounts.reward_mint.to_account_info().key;
    pool_account.reward_vault = ctx.accounts.reward_vault.key();
    pool_account.last_update_time = Clock::get()?.unix_timestamp;
    pool_account.staked_nft = 0;
    pool_account.lock_day = 0;
    pool_account.lock_day_by_class = lock_day_by_class;
    pool_account.reward_policy_by_class = reward_policy_by_class;
    Ok(())
}
