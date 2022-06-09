use crate::{constants::*, states::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct WithdrawSwrd<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    #[account(
        mut,
        seeds = [RS_PREFIX.as_bytes()],
        bump,
        has_one = admin,
    )]
    pub pool_account: Account<'info, PoolConfig>,

    #[account(
        mut,
        seeds = [ RS_VAULT_SEED.as_bytes(), reward_mint.key().as_ref() ],
        bump,
        token::mint = reward_mint,
        token::authority = pool_account,
    )]
    pub reward_vault: Box<Account<'info, TokenAccount>>,

    // send reward to user reward vault
    #[account(
      init_if_needed,
      payer = admin,
      associated_token::mint = reward_mint,
      associated_token::authority = admin
    )]
    reward_to_account: Box<Account<'info, TokenAccount>>,

    // reward mint
    #[account(address = pool_account.reward_mint)]
    reward_mint: Account<'info, Mint>,

    // The Token Program
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle(ctx: Context<WithdrawSwrd>) -> Result<()> {
    let vault_amount = ctx.accounts.reward_vault.amount;

    if vault_amount > 0 {
        let (_pool_account_seed, _bump) =
            Pubkey::find_program_address(&[&(RS_PREFIX.as_bytes())], ctx.program_id);

        // let _bump = ctx.bumps.get(RS_PREFIX).unwrap();
        let pool_seeds = &[RS_PREFIX.as_bytes(), &[_bump]];
        let signer = &[&pool_seeds[..]];

        let token_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.reward_vault.to_account_info().clone(),
            to: ctx.accounts.reward_to_account.to_account_info().clone(),
            authority: ctx.accounts.pool_account.to_account_info().clone(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), token_accounts);
        msg!(
            "Calling the token program to withdraw reward {} to the admin",
            vault_amount
        );
        anchor_spl::token::transfer(cpi_ctx.with_signer(signer), vault_amount)?;
    }
    Ok(())
}
