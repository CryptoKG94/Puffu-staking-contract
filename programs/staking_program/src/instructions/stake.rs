use crate::{constants::*, states::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use std::mem::size_of;

#[derive(Accounts)]
// #[instruction(global_bump: u8, staked_nft_bump: u8)]
pub struct StakeNft<'info> {
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

    #[account(mut)]
    pub user_nft_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
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
    // pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn stake_nft(ctx: Context<StakeNft>, class_id: u32) -> Result<()> {
    let timestamp = Clock::get()?.unix_timestamp;

    // set stake info
    let staking_info = &mut ctx.accounts.nft_stake_info_account;
    staking_info.nft_addr = ctx.accounts.nft_mint.key();
    staking_info.owner = ctx.accounts.owner.key();
    staking_info.stake_time = timestamp;
    staking_info.last_update_time = timestamp;
    staking_info.class_id = class_id;

    // set global info
    ctx.accounts.pool_account.staked_nft += 1;

    // transfer nft to pda (don't need to transfer in this project)
    // let cpi_accounts = Transfer {
    //     from: ctx.accounts.user_nft_token_account.to_account_info(),
    //     to: ctx.accounts.dest_nft_token_account.to_account_info(),
    //     authority: ctx.accounts.owner.to_account_info(),
    // };
    // let token_program = ctx.accounts.token_program.to_account_info();
    // let transfer_ctx = CpiContext::new(token_program, cpi_accounts);
    // token::transfer(transfer_ctx, 1)?;
    Ok(())
}
