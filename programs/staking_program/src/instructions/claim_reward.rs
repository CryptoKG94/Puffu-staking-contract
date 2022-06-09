use crate::{constants::*, error::*, states::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
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
        seeds = [RS_STAKEINFO_SEED.as_ref(), nft_mint.key().as_ref()],
        bump,
    )]
    pub nft_stake_info_account: Account<'info, StakeInfo>,

    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = pool_account,
    )]
    reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = pool_account.reward_mint)]
    pub reward_mint: Account<'info, Mint>,

    // send reward to user reward vault
    #[account(
      init_if_needed,
      payer = owner,
      associated_token::mint = reward_mint,
      associated_token::authority = owner
    )]
    reward_to_account: Box<Account<'info, TokenAccount>>,

    pub nft_mint: Account<'info, Mint>,

    // The Token Program
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> ClaimReward<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.nft_stake_info_account.owner == self.owner.key(),
            StakingError::InvalidUserAddress
        );
        Ok(())
    }
}

#[access_control(ctx.accounts.validate())]
pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let timestamp = Clock::get()?.unix_timestamp;
    let staking_info = &mut ctx.accounts.nft_stake_info_account;

    // calulate reward of this nft
    let pool_account = &mut ctx.accounts.pool_account;
    let reward_per_day = pool_account.reward_policy_by_class[staking_info.class_id as usize];
    // When withdraw nft, calculate and send reward SWRD
    let mut reward: u64 = staking_info.update_reward(timestamp, reward_per_day)?;

    let vault_balance = ctx.accounts.reward_vault.amount;

    if vault_balance < reward {
        reward = vault_balance;
    }

    // Transfer rewards from the pool reward vaults to user reward vaults.
    let (_pool_account_seed, _bump) =
        Pubkey::find_program_address(&[&(RS_PREFIX.as_bytes())], ctx.program_id);
    // let bump = ctx.bumps.get(RS_PREFIX).unwrap();
    let pool_seeds = &[RS_PREFIX.as_bytes(), &[_bump]];
    let signer = &[&pool_seeds[..]];

    let token_program = ctx.accounts.token_program.to_account_info().clone();
    let token_accounts = anchor_spl::token::Transfer {
        from: ctx.accounts.reward_vault.to_account_info().clone(),
        to: ctx.accounts.reward_to_account.to_account_info().clone(),
        authority: ctx.accounts.pool_account.to_account_info().clone(),
    };
    let cpi_ctx = CpiContext::new(token_program, token_accounts);
    msg!(
        "Calling the token program to transfer reward {} to the user",
        reward
    );
    anchor_spl::token::transfer(cpi_ctx.with_signer(signer), reward)?;

    Ok(())
}
