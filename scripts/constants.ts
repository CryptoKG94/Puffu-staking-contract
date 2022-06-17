import { publicKey } from "@project-serum/anchor/dist/cjs/utils";
import { PublicKey } from "@solana/web3.js";

export const RS_PREFIX = "puffu-nft-staking";
export const RS_STAKEINFO_SEED = "puffu-stake-info";
export const RS_STAKE_SEED = "puffu-nft-staking";
export const RS_VAULT_SEED = "puffu-vault";

export const CLASS_TYPES = [10, 20, 50]; // [65, 50, 43, 35, 27, 14, 9, 7, 4];
export const LOCK_DAY = [0, 10, 23]; // 0 => 10, 10 => 10, 23 => 50

export const NETWORK = "devnet";
// devnet
export const SWRD_TOKEN_MINT = new PublicKey(
    "4FkRq5ikN6ZyF2SSH2tgvuFP4kf2vxTuDQN4Kqnz2MQz"
)

export const NFT_CREATOR = new PublicKey(
    "7etbqNa25YWWQztHrwuyXtG39WnAqPszrGRZmEBPvFup"
);

export const PROGRAM_ID = new PublicKey(
    "7RdikeoWp1fzYyw6k1tpoULgZEQ33tFnRE3Nf111NBuu"
)

// mainnet
// export const SWRD_TOKEN_MINT = new PublicKey(
//     "ExLjCck16LmtH87hhCAmTk4RWv7getYQeGhLvoEfDLrH"
// )

// export const NFT_CREATOR = new PublicKey(
//     "6rQse6Jq81nBork8x9UwccJJh4qokVVSYujhQRuQgnna"
// );

// export const PROGRAM_ID = new PublicKey(
//     "6RhXNaW1oQYQmjTc1ypb4bEFe1QasPAgEfFNhQ3HnSqo"
// )