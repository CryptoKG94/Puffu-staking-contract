use crate::constants::*;
use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct StakeInfo {
    pub class_id: u32,
    pub owner: Pubkey,
    pub nft_addr: Pubkey,
    pub stake_time: i64,
    pub last_update_time: i64,
}

impl StakeInfo {
    pub fn update_reward(&mut self, now: i64, reward_per_day: u16) -> Result<u64> {
        let mut last_reward_time = self.last_update_time;
        if last_reward_time < self.stake_time {
            last_reward_time = self.stake_time;
        }

        let unit_amount = (10 as u64).pow(DECIMAL);
        let reward = (unit_amount as u128)
            .checked_mul((now as u128).checked_sub(last_reward_time as u128).unwrap())
            .unwrap()
            .checked_mul(reward_per_day as u128)
            .unwrap()
            .checked_div(DAY as u128)
            .unwrap() as u64;
        // reward = (((now - last_reward_time) / DAY) as u64) * reward_per_day;
        self.last_update_time = now;

        Ok(reward)
        // return Ok(reward);
    }
}
