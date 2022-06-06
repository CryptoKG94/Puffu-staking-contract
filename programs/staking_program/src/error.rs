use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Not Allowed Authority")]
    NotAllowedAuthority,
    #[msg("Invalid User Address")]
    InvalidUserAddress,
    #[msg("Invalid pool number")]
    InvalidPoolError,
    #[msg("No Matching NFT to withdraw")]
    InvalidNFTAddress,
    #[msg("NFT Owner key mismatch")]
    InvalidOwner,
    #[msg("Staking Locked Now")]
    InvalidWithdrawTime,
    #[msg("Withdraw NFT Index OverFlow")]
    IndexOverflow,
    #[msg("Insufficient Lamports")]
    LackLamports,
}
