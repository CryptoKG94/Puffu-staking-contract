use crate::constants::*;
use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct PoolConfig {
    // 1
    pub is_initialized: bool,
    /// admin pubkey
    pub admin: Pubkey,
    /// Paused state of the program
    pub paused: bool,
    /// nft lock period
    pub lock_day: u32,
    /// Mint of the reward token.
    pub reward_mint: Pubkey,
    /// Vault to store reward tokens.
    pub reward_vault: Pubkey,
    /// The last time reward states were updated.
    pub last_update_time: i64,
    /// Tokens Staked
    pub staked_nft: u32,
    /// Reward amount per day according to class type
    pub reward_policy_by_class: [u16; CLASS_TYPES],
    /// lock day according to type
    pub lock_day_by_class: [u16; CLASS_TYPES],
}
