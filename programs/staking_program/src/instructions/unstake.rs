use crate::{constants::*, error::*, states::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct WithdrawNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [RS_PREFIX.as_bytes()],
        bump,
        constraint = pool_account.is_initialized == true,
        constraint = pool_account.paused == false,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = pool_account,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = pool_account.reward_mint)]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [RS_STAKEINFO_SEED.as_ref(), nft_mint.key().as_ref()],
        bump,
        close = owner,
    )]
    pub nft_stake_info_account: Account<'info, StakeInfo>,

    #[account(
        mut,
        // constraint = user_nft_token_account.owner == owner.key()
    )]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [RS_STAKE_SEED.as_ref(), nft_mint.key().as_ref()],
        bump,
    )]
    pub staked_nft_token_account: Account<'info, TokenAccount>,

    // send reward to user reward vault
    #[account(
      init_if_needed,
      payer = owner,
      associated_token::mint = reward_mint,
      associated_token::authority = owner
    )]
    reward_to_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: "nft_mint" is unsafe, but is not documented.
    pub nft_mint: Account<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> WithdrawNft<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.nft_stake_info_account.owner == self.owner.key(),
            StakingError::InvalidUserAddress
        );
        Ok(())
    }
}

#[access_control(ctx.accounts.validate())]
pub fn withdraw_nft(ctx: Context<WithdrawNft>) -> Result<()> {
    let timestamp = Clock::get()?.unix_timestamp;
    let staking_info = &mut ctx.accounts.nft_stake_info_account;
    let pool_account = &mut ctx.accounts.pool_account;

    let lock_day = pool_account.lock_day_by_class[staking_info.class_id as usize];
    let unlock_time = staking_info
        .stake_time
        .checked_add((lock_day as i64).checked_mul(86400 as i64).unwrap())
        .unwrap();

    require!((unlock_time < timestamp), StakingError::InvalidWithdrawTime);

    let reward_per_day = pool_account.reward_policy_by_class[staking_info.class_id as usize];
    // When withdraw nft, calculate and send reward SWRD
    let mut reward: u64 = staking_info.update_reward(timestamp, reward_per_day)?;

    let vault_balance = ctx.accounts.reward_vault.amount;
    if vault_balance < reward {
        reward = vault_balance;
    }

    ctx.accounts.pool_account.staked_nft -= 1;

    // get pool_account seed
    let (_pool_account_seed, _pool_account_bump) =
        Pubkey::find_program_address(&[&(RS_PREFIX.as_bytes())], ctx.program_id);
    let seeds = &[RS_PREFIX.as_bytes(), &[_pool_account_bump]];
    let signer = &[&seeds[..]];
    // let cpi_accounts = Transfer {
    //     from: ctx.accounts.staked_nft_token_account.to_account_info(),
    //     to: ctx.accounts.user_nft_token_account.to_account_info(),
    //     authority: ctx.accounts.pool_account.to_account_info(),
    // };
    // let token_program = ctx.accounts.token_program.to_account_info().clone();
    // let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
    // token::transfer(transfer_ctx, 1)?;

    if reward > 0 {
        let token_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.reward_vault.to_account_info().clone(),
            to: ctx.accounts.reward_to_account.to_account_info().clone(),
            authority: ctx.accounts.pool_account.to_account_info().clone(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token_accounts,
        );
        msg!(
            "Calling the token program to transfer reward {} to the user",
            reward
        );
        anchor_spl::token::transfer(cpi_ctx.with_signer(signer), reward)?;
    }
    Ok(())
}
