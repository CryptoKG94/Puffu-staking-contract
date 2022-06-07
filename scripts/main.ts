import { web3 } from "@project-serum/anchor";
import * as anchor from "@project-serum/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
  Connection,
  clusterApiUrl,
  Commitment
} from "@solana/web3.js";

// for rest function
import {
  TOKEN_PROGRAM_ID,
  AccountLayout,
  MintLayout,
  createMint,
  createAccount,
  mintTo,
} from "@solana/spl-token";

// let bs58 = require("bs58");
import {
  Data,
  updateMetadata,
  Creator,
  createMetadata,
  createMasterEdition,
  getMetadata,
} from "./helpers/metadata";

const BN = anchor.BN;

export const mintNewNFT = async (
  creator: Keypair,
  owner: Keypair,
  index: Number
): Promise<Array<PublicKey>> => {
  const commitment: Commitment = 'processed';
  const network = "https://api.devnet.solana.com/"; // clusterApiUrl("mainnet-beta");
  const connection = new Connection(
    network,
    {
      commitment
    }
  );
  // Create new token mint
  const newMintKey = await createMint(
    connection,
    creator,
    creator.publicKey,
    null,
    0
  );
  const nftAccount = await createAccount(
    connection,
    owner,
    newMintKey,
    owner.publicKey
  );
  await mintTo(
    connection,
    owner,
    newMintKey,
    nftAccount,
    creator.publicKey,
    1
  );

  const name = "RS Test #" + index.toString();
  const metadataUrl = "https://bafybeiadqo2ghp4ltbwfqztl7wabglnd5of3o2nugf72tvmgsb5rxw4voe.ipfs.nftstorage.link/" + index.toString() + ".json";

  const creators = [
    new Creator({
      address: creator.publicKey.toBase58(),
      share: 100,
      verified: true,
    }),
  ];

  let data = new Data({
    name: name,
    symbol: "RST",
    uri: metadataUrl,
    creators,
    sellerFeeBasisPoints: 800,
  });

  let instructions: TransactionInstruction[] = [];

  await createMetadata(
    data,
    creator.publicKey.toBase58(),
    newMintKey.toBase58(),
    creator.publicKey.toBase58(),
    instructions,
    creator.publicKey.toBase58()
  );

  await createMasterEdition(
    new BN(1),
    newMintKey.toBase58(),
    creator.publicKey.toBase58(),
    creator.publicKey.toBase58(),
    creator.publicKey.toBase58(),
    instructions
  );
  const transaction = new Transaction();
  transaction.add(...instructions);
  let txHash = await sendAndConfirmTransaction(
    connection,
    transaction,
    [creator]
  );
  //   console.log("nft creation done tx : ", txHash);
  //   console.log("newMintKey : ", newMintKey.toBase58());
  //   console.log("nftAccount : ", nftAccount.toBase58());
  return [nftAccount, newMintKey];
};

async function main() {
  const MY_WALLET = "/root/.config/solana/id.json";
  const myWallet = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(
      JSON.parse(require("fs").readFileSync(MY_WALLET, "utf8"))
    )
  );

  // console.log("PK : ", myWallet.secretKey.toString());
  let error_count = 0;
  for (let i = 18; i < 100; i++) {
    try {
      await mintNewNFT(myWallet, myWallet, i);
      console.log('current minted: =========> ', i);
    } catch (e) {
      console.log(e);
      error_count++;
      break;
      // continue;
    }
  }
}

main();