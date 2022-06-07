import { PublicKey } from "@solana/web3.js";
import {
    RS_PREFIX,
    RS_STAKEINFO_SEED,
    RS_STAKE_SEED,
    RS_VAULT_SEED,
    PROGRAM_ID,
} from "./constants"

export const getPoolKey = async () => {
    const [poolKey] = await asyncGetPda(
        [Buffer.from(RS_PREFIX)],
        PROGRAM_ID
    );
    return poolKey;
};

export const getRewardVaultKey = async (
    rewardMint: PublicKey
) => {
    const [rewardVaultKey] = await asyncGetPda(
        [
            Buffer.from(RS_VAULT_SEED),
            rewardMint.toBuffer()
        ],
        PROGRAM_ID
    );
    return rewardVaultKey;
};

export const getStakedNFTKey = async (
    nft_mint: PublicKey
) => {
    const [stakedNftKey] = await asyncGetPda(
        [
            Buffer.from(RS_STAKE_SEED),
            nft_mint.toBuffer()
        ],
        PROGRAM_ID
    );
    return stakedNftKey;
};

export const getStakeInfoKey = async (
    nft_mint: PublicKey
) => {
    const [stakedNftKey] = await asyncGetPda(
        [
            Buffer.from(RS_STAKEINFO_SEED),
            nft_mint.toBuffer()
        ],
        PROGRAM_ID
    );
    return stakedNftKey;
};

const asyncGetPda = async (
    seeds: Buffer[],
    programId: PublicKey
): Promise<[PublicKey, number]> => {
    const [pubKey, bump] = await PublicKey.findProgramAddress(seeds, programId);
    return [pubKey, bump];
};
