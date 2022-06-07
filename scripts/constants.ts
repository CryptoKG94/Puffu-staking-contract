import { publicKey } from "@project-serum/anchor/dist/cjs/utils";
import { PublicKey } from "@solana/web3.js";

export const RS_PREFIX = "rs-nft-staking";
export const RS_STAKEINFO_SEED = "rs-stake-info";
export const RS_STAKE_SEED = "rs-nft-staking";
export const RS_VAULT_SEED = "rs-vault";

export const CLASS_TYPES = [65, 50, 43, 35, 27, 14, 9, 7, 4];
export const LOCK_DAY = 20;

export const NETWORK = "mainnet-beta";
// devnet
// export const SWRD_TOKEN_MINT = new PublicKey(
//     "4FkRq5ikN6ZyF2SSH2tgvuFP4kf2vxTuDQN4Kqnz2MQz"
// )

// export const NFT_CREATOR = new PublicKey(
//     "7etbqNa25YWWQztHrwuyXtG39WnAqPszrGRZmEBPvFup"
// );

// export const PROGRAM_ID = new PublicKey(
//     "6RhXNaW1oQYQmjTc1ypb4bEFe1QasPAgEfFNhQ3HnSqo"
// )

// mainnet
export const SWRD_TOKEN_MINT = new PublicKey(
    "ExLjCck16LmtH87hhCAmTk4RWv7getYQeGhLvoEfDLrH"
)

export const NFT_CREATOR = new PublicKey(
    "6rQse6Jq81nBork8x9UwccJJh4qokVVSYujhQRuQgnna"
);

export const PROGRAM_ID = new PublicKey(
    "6RhXNaW1oQYQmjTc1ypb4bEFe1QasPAgEfFNhQ3HnSqo"
)