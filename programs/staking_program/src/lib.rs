use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use std::mem::size_of;

pub mod account;
pub mod constants;
pub mod error;
pub mod utils;

use account::*;
use constants::*;
use error::*;
use utils::*;

declare_id!("FbaMJWS14yAPH68LwFAHxaBSukgBHnAY9VaEfhFxWerb");

#[program]
pub mod staking_program {
    use super::*;

    pub fn initialize_staking_pool(
        ctx: Context<InitializeStakingPool>,
        reward_policy_by_class: [u32; CLASS_TYPES],
    ) -> Result<()> {
        msg!("initializing");

        let pool_account = &mut ctx.accounts.pool_account;

        if pool_account.is_initialized {
            return Err(ProgramError::AccountAlreadyInitialized)?;
        }
        pool_account.is_initialized = true;
        pool_account.admin = *ctx.accounts.admin.key;
        pool_account.paused = true; // initial status is paused
        pool_account.reward_mint = *ctx.accounts.reward_mint.to_account_info().key;
        pool_account.reward_vault = ctx.accounts.reward_vault.key();
        pool_account.last_update_time = Clock::get()?.unix_timestamp;
        pool_account.staked_nft = 0;
        pool_account.reward_policy_by_class = reward_policy_by_class;
        Ok(())
    }

    pub fn stake_nft(ctx: Context<StakeNft>, global_bump: u8, staked_nft_bump: u8) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;

        // set stake info
        let stakingInfo = &mut ctx.accounts.nft_stake_info_account;
        stakingInfo.nft_addr = ctx.accounts.nft_mint.key();
        stakingInfo.owner = ctx.accounts.owner.key();
        stakingInfo.stake_time = timestamp;
        stakingInfo.last_update_time = timestamp;

        // set global info
        ctx.accounts.pool_account.staked_nft += 1;

        // transfer nft to pda
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_nft_token_account.to_account_info(),
            to: ctx.accounts.dest_nft_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let token_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new(token_program, cpi_accounts);
        token::transfer(transfer_ctx, 1)?;
        Ok(())
    }

    #[access_control(user(&ctx.accounts.nft_stake_info_account, &ctx.accounts.owner))]
    pub fn withdraw_nft(
        ctx: Context<WithdrawNft>,
        global_bump: u8,
        stake_info_bump: u8,
    ) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;
        let stakingInfo = &mut ctx.accounts.nft_stake_info_account;

        // When withdraw nft, calculate and send reward SWRD
        let mut reward: u64 = stakingInfo.update_reward(timestamp)?;

        let vault_balance = ctx.accounts.reward_vault.amount;
        if vault_balance < reward {
            reward = vault_balance;
        }

        let lock_day = ctx.accounts.pool_account.lock_day;
        let unlock_time = stakingInfo
            .stake_time
            .checked_add((lock_day as i64).checked_mul(86400 as i64).unwrap())
            .unwrap();

        if unlock_time > timestamp {
            return Err(StakingError::InvalidWithdrawTime.into());
        }

        ctx.accounts.pool_account.staked_nft -= 1;

        // get pool_account seed
        let seeds = &[RS_PREFIX.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.staked_nft_token_account.to_account_info(),
            to: ctx.accounts.user_nft_token_account.to_account_info(),
            authority: ctx.accounts.pool_account.to_account_info(),
        };
        let token_program = ctx.accounts.token_program.to_account_info().clone();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(transfer_ctx, 1)?;

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

    #[access_control(user(&ctx.accounts.nft_stake_info_account, &ctx.accounts.owner))]
    pub fn claim_reward(ctx: Context<ClaimReward>, global_bump: u8) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;
        let stakingInfo = &mut ctx.accounts.nft_stake_info_account;

        // calulate reward of this nft
        let mut reward: u64 = stakingInfo.update_reward(timestamp)?;

        let vault_balance = ctx.accounts.reward_vault.amount;

        if vault_balance < reward {
            reward = vault_balance;
        }

        // Transfer rewards from the pool reward vaults to user reward vaults.
        let pool_seeds = &[RS_PREFIX.as_bytes(), &[global_bump]];
        let signer = &[&pool_seeds[..]];

        let token_program = ctx.accounts.token_program.clone();
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

    pub fn deposit_swrd(ctx: Context<DepositSwrd>, amount: u64) -> Result<()> {
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

    pub fn withdraw_swrd(ctx: Context<WithdrawSwrd>, global_bump: u8) -> Result<()> {
        let vault_amount = ctx.accounts.reward_vault.amount;

        if vault_amount > 0 {
            let pool_seeds = &[RS_PREFIX.as_bytes(), &[global_bump]];
            let signer = &[&pool_seeds[..]];

            let token_accounts = anchor_spl::token::Transfer {
                from: ctx.accounts.reward_vault.to_account_info().clone(),
                to: ctx.accounts.funder_account.to_account_info().clone(),
                authority: ctx.accounts.pool_account.to_account_info().clone(),
            };
            let cpi_ctx =
                CpiContext::new(ctx.accounts.token_program.to_account_info(), token_accounts);
            msg!(
                "Calling the token program to withdraw reward {} to the admin",
                vault_amount
            );
            anchor_spl::token::transfer(cpi_ctx.with_signer(signer), vault_amount)?;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeStakingPool<'info> {
    // #[account(zero)]
    // pub pool_account: AccountLoader<'info, UserPool>,

    // #[account(mut)]
    // pub authority: Signer<'info>,

    // The pool authority
    #[account(mut, signer)]
    admin: AccountInfo<'info>,

    #[account(
        init,
        seeds = [RS_PREFIX.as_bytes()],
        bump,
        payer = admin,
        space = POOL_CONFIG_SIZE,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    // reward mint
    reward_mint: AccountInfo<'info>,

    // reward vault that holds the reward mint for distribution
    #[account(
        init,
        token::mint = reward_mint,
        token::authority = pool_account,
        seeds = [ RS_VAULT_SEED.as_bytes(), admin.key.as_ref(), reward_mint.key.as_ref() ],
        bump,
        payer = admin,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    // The rent sysvar
    rent: Sysvar<'info, Rent>,
    // system program
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,

    // token program
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
// #[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct StakeNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut,
        constraint = pool_account.is_initialized == true,
        constraint = pool_account.paused == false,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    #[account(mut)]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        seeds = [RS_STAKE_SEED.as_ref(), nft_mint.key.as_ref()],
        bump,
        token::mint = nft_mint,
        token::authority = pool_account,
    )]
    pub dest_nft_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = owner,
        seeds = [RS_STAKEINFO_SEED.as_ref(), nft_mint.key.as_ref()],
        bump,
        space = 8 + size_of::<StakeInfo>(),
    )]
    pub nft_stake_info_account: Account<'info, StakeInfo>,

    /// CHECK: unsafe
    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8, stake_info_bump: u8, staked_nft_bump: u8)]
pub struct WithdrawNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut,
        constraint = pool_account.is_initialized == true,
        constraint = pool_account.paused == false,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    #[account(mut)]
    reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [RS_STAKEINFO_SEED.as_ref(), nft_mint.key.as_ref()],
        bump = stake_info_bump,
        close = owner,
    )]
    pub nft_stake_info_account: Account<'info, StakeInfo>,

    #[account(
        mut,
        constraint = user_nft_token_account.owner == owner.key()
    )]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [RS_STAKE_SEED.as_ref(), nft_mint.key.as_ref()],
        bump = staked_nft_bump
    )]
    pub staked_nft_token_account: Account<'info, TokenAccount>,

    // send reward to user reward vault
    #[account(mut)]
    reward_to_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: "nft_mint" is unsafe, but is not documented.
    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8, stake_info_bump: u8, staked_nft_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut,
        constraint = pool_account.is_initialized == true,
        constraint = pool_account.paused == false,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    #[account(
        seeds = [RS_STAKEINFO_SEED.as_ref(), nft_mint.key.as_ref()],
        bump = stake_info_bump,
    )]
    pub nft_stake_info_account: Account<'info, StakeInfo>,

    #[account(mut)]
    reward_vault: Box<Account<'info, TokenAccount>>,

    // send reward to user reward vault
    #[account(mut)]
    reward_to_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: "nft_mint" is unsafe, but is not documented.
    pub nft_mint: AccountInfo<'info>,

    // The Token Program
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
    // #[account(
    //     mut,
    //     seeds = [POOL_WALLET_SEED.as_ref()],
    //     bump = pool_wallet_bump,
    // )]
    // /// CHECK: "pool_wallet" is unsafe, but is not documented.
    // pub pool_wallet: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSwrd<'info> {
    #[account(mut, signer)]
    funder: AccountInfo<'info>,
    #[account(mut)]
    reward_vault: Box<Account<'info, TokenAccount>>,

    // funder account
    #[account(mut)]
    funder_account: Account<'info, TokenAccount>,

    // The Token Program
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8, stake_info_bump: u8, vault_bump: u8)]
pub struct WithdrawSwrd<'info> {
    #[account(mut, signer)]
    authority: AccountInfo<'info>,
    #[account(mut,
        constraint = pool_account.is_initialized == true,
        constraint = pool_account.paused == false,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    #[account(
        mut,
        seeds = [ RS_VAULT_SEED.as_bytes(), pool_account.key().as_ref(), authority.key.as_ref(), reward_mint.key.as_ref() ],
        bump = vault_bump,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    // funder account
    #[account(mut)]
    pub funder_account: Account<'info, TokenAccount>,

    // reward mint
    reward_mint: AccountInfo<'info>,

    // The Token Program
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
}

// Access control modifiers
fn user(stake_info_account: &Account<StakeInfo>, user: &AccountInfo) -> Result<()> {
    require!(
        stake_info_account.owner == *user.key,
        StakingError::InvalidUserAddress
    );
    Ok(())
}
