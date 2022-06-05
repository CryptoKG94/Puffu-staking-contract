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
        pool_account.authority = *ctx.accounts.authority.key;
        pool_account.paused = true; // initial status is paused
        pool_account.reward_mint = *ctx.accounts.reward_mint.to_account_info().key;
        pool_account.reward_vault = ctx.accounts.reward_vault.key();
        pool_account.last_update_time = Clock::get()?.unix_timestamp;
        pool_account.staked_nft = 0;
        pool_account.reward_policy_by_class = reward_policy_by_class;
        Ok(())
    }

    #[access_control(user(&ctx.accounts.pool_account, &ctx.accounts.owner))]
    pub fn stake_nft(ctx: Context<StakeNft>, global_bump: u8, staked_nft_bump: u8) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;

        // let staked_item = StakedNFT {
        //     nft_addr: ctx.accounts.nft_mint.key(),
        //     stake_time: timestamp,
        // };

        let mut staking_pool = ctx.accounts.pool_account.load_mut()?;
        staking_pool.add_nft(staked_item);

        ctx.accounts.global_authority.fixed_nft_count += 1;

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

    #[access_control(user(&ctx.accounts.pool_account, &ctx.accounts.owner))]
    pub fn withdraw_nft(
        ctx: Context<WithdrawNft>,
        global_bump: u8,
        staked_nft_bump: u8,
        pool_wallet_bump: u8,
    ) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;
        let mut staking_pool = ctx.accounts.pool_account.load_mut()?;
        let reward: u64 = staking_pool.remove_nft(
            ctx.accounts.owner.key(),
            ctx.accounts.nft_mint.key(),
            timestamp,
        )?;

        staking_pool.pending_reward += reward;

        ctx.accounts.global_authority.fixed_nft_count -= 1;

        let seeds = &[GLOBAL_AUTHORITY_SEED.as_bytes(), &[global_bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.staked_nft_token_account.to_account_info(),
            to: ctx.accounts.user_nft_token_account.to_account_info(),
            authority: ctx.accounts.global_authority.to_account_info(),
        };
        let token_program = ctx.accounts.token_program.to_account_info();
        let transfer_ctx = CpiContext::new_with_signer(token_program, cpi_accounts, signer);
        token::transfer(transfer_ctx, 1)?;
        /*
                sol_transfer_with_signer(
                    ctx.accounts.pool_wallet.to_account_info(),
                    ctx.accounts.owner.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                    &[&[POOL_WALLET_SEED.as_ref(), &[pool_wallet_bump]]],
                    reward
                )?;
        */
        Ok(())
    }

    #[access_control(user(&ctx.accounts.pool_account, &ctx.accounts.owner))]
    pub fn claim_reward(
        ctx: Context<ClaimReward>,
        global_bump: u8,
        staked_nft_bump: u8,
        pool_wallet_bump: u8,
    ) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;
        let mut staking_pool = ctx.accounts.pool_account.load_mut()?;
        let reward: u64 = staking_pool.claim_reward(timestamp)?;

        // // send native currency(SOL)
        // if ctx.accounts.pool_wallet.to_account_info().lamports() < 1000_000_000 + reward {
        //     return Err(StakingError::LackLamports.into());
        // }

        // sol_transfer_with_signer(
        //     ctx.accounts.pool_wallet.to_account_info(),
        //     ctx.accounts.owner.to_account_info(),
        //     ctx.accounts.system_program.to_account_info(),
        //     &[&[POOL_WALLET_SEED.as_ref(), &[pool_wallet_bump]]],
        //     reward,
        // )?;

        let vault_balance = ctx.accounts.reward_vault.amount;

        // settle pending reward
        // ctx.accounts.user_account.reward_earned_pending = 0;
        // ctx.accounts.user_account.reward_earned_claimed = ctx.accounts.user_account.reward_earned_claimed + reward_amount;

        if vault_balance < reward {
            reward = vault_balance;
        }

        // Transfer rewards from the pool reward vaults to user reward vaults.
        let (_pool_pda, pool_bump) = Pubkey::find_program_address(
            &[
                RS_PREFIX.as_bytes(),
                ctx.accounts.pool_account.authority.as_ref(),
                ctx.accounts.pool_account.config.as_ref(),
            ],
            ctx.program_id,
        );
        let seeds = &[
            RS_PREFIX.as_bytes(),
            ctx.accounts.pool_account.authority.as_ref(),
            ctx.accounts.pool_account.config.as_ref(),
            &[pool_bump],
        ]; // need this to sign the pda, match the authority

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
        anchor_spl::token::transfer(cpi_ctx.with_signer(&[&seeds[..]]), reward)?;

        Ok(())
    }

    pub fn deposit_swrd(ctx: Context<DepositSwrd>, amount: u64) -> Result<()> {
        // Transfer reward tokens into the vault.
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.funder_vault.to_account_info(),
                to: ctx.accounts.reward_vault.to_account_info(),
                authority: ctx.accounts.funder.to_account_info(),
            },
        );

        anchor_spl::token::transfer(cpi_ctx, amount)?;

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
    authority: AccountInfo<'info>,

    #[account(
        init,
        seeds = [RS_PREFIX.as_bytes(), authority.key.as_ref()],
        bump,
        payer = authority,
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
        seeds = [ RS_PREFIX.as_bytes(), pool_account.key().as_ref(), authority.key.as_ref(), reward_mint.key.as_ref() ],
        bump,
        payer = authority,
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
#[instruction(global_bump: u8, staked_nft_bump: u8)]
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
        seeds = [RS_PREFIX.as_ref(), nft_mint.key.as_ref()],
        bump,
        space = 8 + size_of::<StakeInfo>(),
        // token::mint = nft_mint,
        // token::authority = pool_account.key().as_ref(),
    )]
    pub dest_nft_token_account: Account<'info, StakeInfo>,

    /// CHECK: unsafe
    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8, pool_wallet_bump: u8)]
pub struct WithdrawNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool_account: AccountLoader<'info, UserPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        seeds = [POOL_WALLET_SEED.as_ref()],
        bump = pool_wallet_bump,
    )]
    /// CHECK: "pool_wallet" is unsafe, but is not documented.    
    pub pool_wallet: AccountInfo<'info>,

    #[account(
        mut,
        constraint = user_nft_token_account.owner == owner.key()
    )]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = ["staked-nft".as_ref(), nft_mint.key.as_ref()],
        bump = staked_nft_bump
    )]
    pub staked_nft_token_account: Account<'info, TokenAccount>,

    /// CHECK: "nft_mint" is unsafe, but is not documented.
    pub nft_mint: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(global_bump: u8, staked_nft_bump: u8, pool_wallet_bump: u8)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool_account: AccountLoader<'info, UserPool>,

    #[account(mut)]
    reward_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_ref()],
        bump = global_bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    // send reward to user reward vault
    #[account(mut)]
    reward_to_account: Box<Account<'info, TokenAccount>>,

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
    // Pool owner
    authority: AccountInfo<'info>,

    #[account(mut, signer)]
    funder: AccountInfo<'info>,
    #[account(mut)]
    reward_vault: Box<Account<'info, TokenAccount>>,

    // funder vault
    #[account(mut)]
    funder_vault: Account<'info, TokenAccount>,

    // The Token Program
    #[account(address = spl_token::id())]
    token_program: AccountInfo<'info>,
}

// Access control modifiers
fn user(pool_loader: &AccountLoader<UserPool>, user: &AccountInfo) -> Result<()> {
    let user_pool = pool_loader.load()?;
    require!(user_pool.owner == *user.key, StakingError::InvalidUserPool);
    Ok(())
}
