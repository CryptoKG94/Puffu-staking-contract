use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::*;

pub const POOL_CONFIG_SIZE: usize = 8 + // discriminator
    1 + // is_initialized
    32 + // authority
    1 + // paused
    32 + // reward_mint
    32 + // reward_vault
    8 + // last_update_time
    4 + // staked_nft
    4 * CLASS_TYPES; // reward_policy_by_class

#[account]
#[derive(Default)]
pub struct PoolConfig {
    // 1
    pub is_initialized: bool,
    /// authority (owner) pubkey
    pub authority: Pubkey,
    /// Paused state of the program
    pub paused: bool,
    /// Mint of the reward token.
    pub reward_mint: Pubkey,
    /// Vault to store reward tokens.
    pub reward_vault: Pubkey,
    /// The last time reward states were updated.
    pub last_update_time: i64,
    /// Tokens Staked
    pub staked_nft: u32,
    /// Reward amount per day according to class type
    pub reward_policy_by_class: [u32; CLASS_TYPES],
}

#[account]
#[derive(Default)]
pub struct StakeInfo {
    pub class_id: u32,
}

#[zero_copy]
#[derive(Default)]
pub struct Item {
    // 72
    pub owner: Pubkey,    // 32
    pub nft_addr: Pubkey, // 32
    pub stake_time: i64,  // 8
}

#[zero_copy]
#[derive(Default, PartialEq)]
pub struct StakedNFT {
    pub nft_addr: Pubkey, // 32
    pub stake_time: i64,  // 8
}

#[account(zero_copy)]
pub struct UserPool {
    // 2064
    pub owner: Pubkey,                           // 32
    pub item_count: u64,                         // 8
    pub items: [StakedNFT; NFT_STAKE_MAX_COUNT], // 40 * 50 = 2000
    pub reward_time: i64,                        // 8
    pub pending_reward: u64,                     // 8
}
impl Default for UserPool {
    #[inline]
    fn default() -> UserPool {
        UserPool {
            owner: Pubkey::default(),
            item_count: 0,
            items: [StakedNFT {
                ..Default::default()
            }; NFT_STAKE_MAX_COUNT],
            reward_time: 0,
            pending_reward: 0,
        }
    }
}

impl UserPool {
    pub fn add_nft(&mut self, item: StakedNFT) {
        self.items[self.item_count as usize] = item;
        self.item_count += 1;
    }
    pub fn remove_nft(&mut self, owner: Pubkey, nft_mint: Pubkey, now: i64) -> Result<u64> {
        require!(self.owner.eq(&owner), StakingError::InvalidOwner);
        let mut withdrawn: u8 = 0;
        let mut reward: u64 = 0;
        for i in 0..self.item_count {
            let index = i as usize;
            if self.items[index].nft_addr.eq(&nft_mint) {
                //require!(self.items[index].stake_time + LIMIT_PERIOD <= now, StakingError::InvalidWithdrawTime);
                let mut last_reward_time = self.reward_time;
                if last_reward_time < self.items[index].stake_time {
                    last_reward_time = self.items[index].stake_time;
                }
                reward = (((now - last_reward_time) / DAY) as u64) * REWARD_PER_DAY;
                // remove nft
                if i != self.item_count - 1 {
                    let last_idx = self.item_count - 1;
                    self.items[index] = self.items[last_idx as usize];
                }
                self.item_count -= 1;
                withdrawn = 1;
                break;
            }
        }
        require!(withdrawn == 1, StakingError::InvalidNFTAddress);
        Ok(reward)
    }
    pub fn claim_reward(&mut self, now: i64) -> Result<u64> {
        let mut total_reward: u64 = 0;
        for i in 0..self.item_count {
            let index = i as usize;
            //require!(self.items[index].stake_time + LIMIT_PERIOD <= now, StakingError::InvalidWithdrawTime);
            let mut last_reward_time = self.reward_time;
            if last_reward_time < self.items[index].stake_time {
                last_reward_time = self.items[index].stake_time;
            }
            let reward = (((now - last_reward_time) / DAY) as u64) * REWARD_PER_DAY;
            total_reward += reward;
        }
        total_reward += self.pending_reward;
        self.pending_reward = 0;
        self.reward_time = now;
        Ok(total_reward)
    }
}
