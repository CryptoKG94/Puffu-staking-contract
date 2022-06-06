pub const CLASS_TYPES: usize = 9;

pub const NFT_STAKE_MAX_COUNT: usize = 50;
pub const NFT_TOTAL_COUNT: usize = 5000;

pub const GLOBAL_AUTHORITY_SEED: &str = "global-authority";
pub const RS_PREFIX: &str = "rs-nft-staking";
pub const RS_STAKEINFO_SEED: &str = "rs-stake-info";
pub const RS_STAKE_SEED: &str = "rs-nft-staking";
pub const RS_VAULT_SEED: &str = "rs-vault";

pub const DAY: i64 = 60 * 1; //60 * 60 * 24; // 1 mins
pub const LIMIT_PERIOD: i64 = 60 * 10; //DAY * 15;
pub const REWARD_PER_DAY: u64 = 2 * 10_000_000;
