import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { RsStakingProgram } from '../target/types/rs_staking_program';
import { SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { Token, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";
import bs58 from 'bs58';
import { ASSOCIATED_PROGRAM_ID } from '@project-serum/anchor/dist/cjs/utils/token';

const PublicKey = anchor.web3.PublicKey;

const SWRD_DECIMAL = 6;
const GLOBAL_AUTHORITY_SEED = "global-authority";
const RS_PREFIX = "rs-nft-staking";
const RS_STAKEINFO_SEED = "rs-stake-info";
const RS_STAKE_SEED = "rs-nft-staking";
const RS_VAULT_SEED = "rs-vault";


describe('staking_program', () => {

  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RsStakingProgram as Program<RsStakingProgram>;
  const superOwner = anchor.web3.Keypair.generate();
  const user = anchor.web3.Keypair.generate();

  // const USER_POOL_SIZE = 2064;
  // const GLOBAL_POOL_SIZE = 360_016;

  let nft_token_mint = null as Token;
  let reward_mint = null as Token;
  let user_nft_token_account = null;
  let user_reward_account = null;
  let funder_vault_account = null;

  let initial_reward_vault_amount = 1_000_000_000;
  let deposit_amount = 100_000_000;
  let class_types = [
    10,
    25,
    30,
    40,
    55,
    60,
    70,
    80,
    90,
  ];
  let lock_day = 0;

  it('Is initialized!', async () => {
    // Add your test here.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(superOwner.publicKey, 9000000000),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, 9000000000),
      "confirmed"
    );

    console.log("super owner =", superOwner.publicKey.toBase58());
    console.log("user =", user.publicKey.toBase58());

    // nft mint
    nft_token_mint = await Token.createMint(
      provider.connection,
      user,
      superOwner.publicKey,
      null,
      0,
      TOKEN_PROGRAM_ID
    );
    user_nft_token_account = await nft_token_mint.createAccount(user.publicKey);

    await nft_token_mint.mintTo(
      user_nft_token_account,
      superOwner,
      [],
      1
    );

    console.log("create nft token!");

    // token mint
    reward_mint = await Token.createMint(
      provider.connection,
      user,
      superOwner.publicKey,
      null,
      SWRD_DECIMAL,
      TOKEN_PROGRAM_ID
    );

    // user_reward_account = await reward_mint.createAccount(user.publicKey);
    // funder_vault_account = await reward_mint.createAccount(superOwner.publicKey);

    // associate test
    user_reward_account = await reward_mint.createAssociatedTokenAccount(user.publicKey);
    funder_vault_account = await reward_mint.createAssociatedTokenAccount(superOwner.publicKey);

    user_reward_account = (await PublicKey.findProgramAddress(
      [
        user.publicKey.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        reward_mint.publicKey.toBuffer(), // mint address
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];

    funder_vault_account = (await PublicKey.findProgramAddress(
      [
        superOwner.publicKey.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        reward_mint.publicKey.toBuffer(), // mint address
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];
    // console.log("funder_vault_account_ass = ", funder_vault_account_ass);
    console.log("funder_vault_account = ", funder_vault_account);
    // console.log("associatedTokenAccountPubkey = ", associatedTokenAccountPubkey);
    // test end

    await reward_mint.mintTo(
      funder_vault_account,
      superOwner,
      [],
      initial_reward_vault_amount
    )

    console.log("create reward token!");

    let _funder_vault_account = await reward_mint.getAccountInfo(funder_vault_account);
    let _user_nft_token_account = await nft_token_mint.getAccountInfo(user_nft_token_account);

    assert.ok(Number(_funder_vault_account.amount) == initial_reward_vault_amount);
    assert.ok(Number(_user_nft_token_account.amount) == 1);

    // create PDAs
    const [pool_account_pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(RS_PREFIX)],
      program.programId
    );

    console.log("pool_account =", pool_account_pda.toBase58());

    const [vault_pda, walletBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_VAULT_SEED),
        reward_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    console.log("vault_key =", vault_pda.toBase58());

    const res = await program.methods.initializeStakingPool(class_types, lock_day).accounts({
      admin: superOwner.publicKey,
      poolAccount: pool_account_pda,
      rewardMint: reward_mint.publicKey,
      rewardVault: vault_pda,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY
    }).signers([superOwner]).rpc();

    let _pool_config = await program.account.poolConfig.fetch(pool_account_pda);

    console.log("superOwner from contract = ", _pool_config.admin.toBase58());
    assert.ok(_pool_config.admin.toBase58() == superOwner.publicKey.toBase58());
    assert.ok(_pool_config.rewardMint.equals(reward_mint.publicKey));
    assert.ok(_pool_config.rewardVault.equals(vault_pda));
    console.log("second class id: ", _pool_config.rewardPolicyByClass[1]);
    assert.ok(_pool_config.rewardPolicyByClass[1] == class_types[1]);
    assert.ok(_pool_config.stakedNft == 0);

    console.log("Your transaction signature", res);
  });

  it("deposit reward", async () => {
    // create PDAs
    const [pool_account_pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(RS_PREFIX)],
      program.programId
    );

    const [vault_pda, walletBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_VAULT_SEED),
        reward_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const ix = await program.methods.depositSwrd(
      new anchor.BN(deposit_amount)
    ).accounts({
      funder: superOwner.publicKey,
      rewardVault: vault_pda,
      funderAccount: funder_vault_account,
      poolAccount: pool_account_pda,
      rewardMint: reward_mint.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([superOwner]).rpc();

    let _vault_account = await reward_mint.getAccountInfo(vault_pda);
    assert.ok(Number(_vault_account.amount) == deposit_amount);
    let _funder_account = await reward_mint.getAccountInfo(funder_vault_account);
    console.log("deposit amount: ", Number(_vault_account.amount));
    console.log("remain amount: ", Number(_funder_account.amount));
    console.log("Your transaction signature", ix);
  })

  it("Stake Nft", async () => {
    const [pool_account_pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(RS_PREFIX)],
      program.programId
    );

    const [staked_nft_pda, staked_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_STAKE_SEED),
        nft_token_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [stake_info_pda, stake_info_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_STAKEINFO_SEED),
        nft_token_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const ix = await program.methods.stakeNft(2).accounts({
      owner: user.publicKey,
      poolAccount: pool_account_pda,
      nftMint: nft_token_mint.publicKey,
      userNftTokenAccount: user_nft_token_account,
      destNftTokenAccount: staked_nft_pda,
      nftStakeInfoAccount: stake_info_pda,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
      .signers([user])
      .rpc();
    console.log("Your transaction signature", ix);

    let _stakeinfo = await program.account.stakeInfo.fetch(stake_info_pda);
    assert.ok(_stakeinfo.owner.equals(user.publicKey));
    assert.ok(_stakeinfo.nftAddr.equals(nft_token_mint.publicKey));
    console.log("class_id: ", _stakeinfo.classId);
    console.log("stake_time : ", _stakeinfo.stakeTime);

    let _destNftTokenAccount = await nft_token_mint.getAccountInfo(staked_nft_pda);
    assert.ok(Number(_destNftTokenAccount.amount) == 1);

    let _user_nft_token_account = await nft_token_mint.getAccountInfo(user_nft_token_account);
    assert.ok(Number(_user_nft_token_account.amount) == 0);

  })

  it("Claim reward", async () => {
    const [pool_account_pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(RS_PREFIX)],
      program.programId
    );

    const [vault_pda, walletBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_VAULT_SEED),
        reward_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [stake_info_pda, stake_info_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_STAKEINFO_SEED),
        nft_token_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const ix = await program.methods.claimReward().accounts({
      owner: user.publicKey,
      poolAccount: pool_account_pda,
      rewardMint: reward_mint.publicKey,
      nftMint: nft_token_mint.publicKey,
      nftStakeInfoAccount: stake_info_pda,
      rewardToAccount: user_reward_account,
      rewardVault: vault_pda,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
      .signers([user])
      .rpc();
    console.log("Your transaction signature", ix);

    let _user_reward_account = await reward_mint.getAccountInfo(user_reward_account);
    console.log("reward amount: ", Number(_user_reward_account.amount));

  })

  it("Withdraw Nft", async () => {
    const [pool_account_pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(RS_PREFIX)],
      program.programId
    );

    const [vault_pda, walletBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_VAULT_SEED),
        reward_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [staked_nft_pda, staked_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_STAKE_SEED),
        nft_token_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [stake_info_pda, stake_info_bump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_STAKEINFO_SEED),
        nft_token_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const ix = await program.methods.withdrawNft().accounts({
      owner: user.publicKey,
      poolAccount: pool_account_pda,
      nftMint: nft_token_mint.publicKey,
      userNftTokenAccount: user_nft_token_account,
      stakedNftTokenAccount: staked_nft_pda,
      nftStakeInfoAccount: stake_info_pda,
      rewardToAccount: user_reward_account,
      rewardVault: vault_pda,
      rewardMint: reward_mint.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
      .signers([user])
      .rpc();
    console.log("Your transaction signature", ix);

    let _user_nft_token_account = await nft_token_mint.getAccountInfo(user_nft_token_account);
    assert.ok(Number(_user_nft_token_account.amount) == 1);
    let _user_reward_account = await reward_mint.getAccountInfo(user_reward_account);
    console.log("reward amount: ", Number(_user_reward_account.amount));

  })

  it("withdraw reward", async () => {
    const [vault_pda, walletBump] = await PublicKey.findProgramAddress(
      [
        Buffer.from(RS_VAULT_SEED),
        reward_mint.publicKey.toBuffer(),
      ],
      program.programId
    );

    const [pool_account_pda, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(RS_PREFIX)],
      program.programId
    );

    const ix = await program.methods.withdrawSwrd().accounts({
      admin: superOwner.publicKey,
      poolAccount: pool_account_pda,
      rewardVault: vault_pda,
      rewardToAccount: funder_vault_account,
      rewardMint: reward_mint.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    }).signers([superOwner]).rpc();

    let _vault_account = await reward_mint.getAccountInfo(vault_pda);
    assert.ok(Number(_vault_account.amount) == 0);
    let _funder_account = await reward_mint.getAccountInfo(funder_vault_account);
    console.log("deposit amount: ", Number(_vault_account.amount));
    console.log("remain amount: ", Number(_funder_account.amount));
    console.log("Your transaction signature", ix);
  })
});
