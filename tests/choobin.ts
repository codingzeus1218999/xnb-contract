import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Choobin } from "../target/types/choobin";

import * as fs from "fs";
import BN from "bn.js";

import {
  PublicKey,
  Connection,
  SYSVAR_RENT_PUBKEY,
  Keypair,
  Secp256k1Program,
  LAMPORTS_PER_SOL,
  Transaction,
  sendAndConfirmTransaction,
  Message,
  SYSVAR_RECENT_BLOCKHASHES_PUBKEY
} from "@solana/web3.js";

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createInitializeMintInstruction,
  createAssociatedTokenAccountInstruction,
  getOrCreateAssociatedTokenAccount
} from "@solana/spl-token";


async function findAssociatedTokenAddress(
  walletAddress: PublicKey,
  tokenMintAddress: PublicKey
): Promise<PublicKey> {
  return (await PublicKey.findProgramAddress(
    [
      walletAddress.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      tokenMintAddress.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  ))[0];
};

describe("choobin", async () => {

  const NETWORK = "http://localhost:8899";
  // const NETWORK = "https://api.devnet.solana.com";
  // const NETWORK = "https://api.mainnet-beta.solana.com";
  const connection = new Connection(NETWORK, "confirmed");

  //--------- Admin wallet -----------------------
  const pk_admin = Uint8Array.from(
    JSON.parse(fs.readFileSync(`./keys/admin.json`) as unknown as string)
  );
  const adminKeypair = Keypair.fromSecretKey(pk_admin);
  console.log("Admin address:", adminKeypair.publicKey.toBase58());

  //--------- User wallet -----------------------
  const pk_user = Uint8Array.from(
    JSON.parse(fs.readFileSync(`./keys/user.json`) as unknown as string)
  );
  const userKeypair = Keypair.fromSecretKey(pk_user);
  console.log("User address:", userKeypair.publicKey.toBase58());

  const program = anchor.workspace.Choobin as Program<Choobin>;
  console.log("program ID: ", program.programId.toBase58())

  // const mintKey: anchor.web3.Keypair = anchor.web3.Keypair.generate();
  // console.log("mint: ", mintKey.publicKey.toBase58())

  //---------- Choobin Mint ------------------------
  const mint_pubkey = new PublicKey("GKoJRfDxq3mJg8YeQTNfPWnuTSuvNEaUu3pzpJM7XZbp");
  console.log("choobin token: ", mint_pubkey.toBase58())

  //---------- Client's wallet ---------------------
  const treasury_pubkey = adminKeypair.publicKey;
  console.log("treasury_pubkey: ", treasury_pubkey.toBase58())

  //---------- SEEDS -----------------------------
  const PRESALE_INFO_SEED = "presale_info";
  const USER_INFO_SEED = "user_info";

  //--------- PDAs --------------------------------
  const presale_info = await PublicKey.findProgramAddress(
    [
      Buffer.from(PRESALE_INFO_SEED),
    ],
    program.programId
  );
  console.log("presale_info : ", presale_info[0].toBase58());

  const user_info = await PublicKey.findProgramAddress(
    [
      Buffer.from(USER_INFO_SEED),
      userKeypair.publicKey.toBuffer()
    ],
    program.programId
  );
  console.log("user_info : ", user_info[0].toBase58());
  //---------- ATAs --------------------------------
  const admin_mint_ata = await findAssociatedTokenAddress(
    adminKeypair.publicKey,
    mint_pubkey
  );
  console.log('admin_mint_ata:', admin_mint_ata.toBase58());

  const user_mint_ata = await findAssociatedTokenAddress(
    userKeypair.publicKey,
    mint_pubkey
  );
  console.log('user_mint_ata:', user_mint_ata.toBase58());

  const presale_info_mint_ata = await findAssociatedTokenAddress(
    presale_info[0],
    mint_pubkey
  );
  console.log('presale_info_mint_ata:', presale_info_mint_ata.toBase58());

  const user_info_mint_ata = await findAssociatedTokenAddress(
    user_info[0],
    mint_pubkey
  );
  console.log('user_info_mint_ata:', user_info_mint_ata.toBase58());

  //---------- Functions -------------------------
  async function fetchPresaleInfo() {
    console.log("---------- Presale Info -----------------")
    let presaleInfo = await program.account.presaleInfo.fetch(presale_info[0]);
    console.log("is_initialized: ", presaleInfo.isInitialized);
    console.log("admin: ", presaleInfo.admin.toBase58());
    console.log("mint: ", presaleInfo.mint.toBase58());
    console.log("amount: ", presaleInfo.amount.toString());
    console.log("price: ", presaleInfo.price.toString());
    console.log("end_timestamp: ", presaleInfo.endTimestamp.toString());
    console.log("treasury: ", presaleInfo.treasury.toBase58());
  };

  async function fetchUserInfo() {
    console.log("---------- User Info -----------------")
    let userInfo = await program.account.userInfo.fetch(user_info[0]);
    console.log("is_initialized: ", userInfo.isInitialized);
    console.log("admin: ", userInfo.admin.toBase58());
    console.log("amount: ", userInfo.amount.toString());
  };

  // it("Is initialized!", async () => {
  //   const tx = await program.methods
  //   .initialize()
  //   .accounts({
  //     presaleInfo: presale_info[0],
  //     initializer: adminKeypair.publicKey,
  //     mint: mint_pubkey,
  //     treasury: treasury_pubkey,
  //     systemProgram: anchor.web3.SystemProgram.programId,
  //   })
  //   .signers([adminKeypair])
  //   .rpc();
  //   console.log("Your transaction signature", tx);

  //   //-------- Fetch data --------------
  //   await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  //   await fetchPresaleInfo();
  // });

  // it("deposit choobin to presale smart contract", async () => {
  //   const amount = new BN("562500000000000000");  //562500000 * 1e9
  //   // console.log("deposit amount: ", amount.toArray("le", 8))
  //   const tx = await program.methods
  //     .depositToken(amount)
  //     .accounts({
  //       presaleInfo: presale_info[0],
  //       presaleInfoMintAta: presale_info_mint_ata,
  //       payer: adminKeypair.publicKey,
  //       payerMintAta: admin_mint_ata,
  //       mint: mint_pubkey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
  //     })
  //     .signers([adminKeypair])
  //     .rpc();
  //   console.log("Your transaction signature", tx);

  //   //-------- Fetch data --------------
  //   await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  //   await fetchPresaleInfo();
  // });

  // it("set end_timestamp", async () => {
  //   const feature_endtimestamp = new BN(1724976000);  // 2024/8/30
  //   const old_endtimestamp = new BN(1711756800);  // 2024/3/30
  //   const endtimestamp = old_endtimestamp;

  //   const tx = await program.methods
  //     .setEndtime(endtimestamp)
  //     .accounts({
  //       presaleInfo: presale_info[0],
  //       admin: adminKeypair.publicKey
  //     })
  //     .signers([adminKeypair])
  //     .rpc();
  //   console.log("Your transaction signature", tx);

  //   //-------- Fetch data --------------
  //   await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  //   await fetchPresaleInfo();
  // });

  // it("burn choobin from presale smart contract", async () => {
  //   // console.log("deposit amount: ", amount.toArray("le", 8))
  //   const tx = await program.methods
  //     .burnToken()
  //     .accounts({
  //       presaleInfo: presale_info[0],
  //       presaleInfoMintAta: presale_info_mint_ata,
  //       user: adminKeypair.publicKey,
  //       mint: mint_pubkey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
  //     })
  //     .signers([adminKeypair])
  //     .rpc();
  //   console.log("Your transaction signature", tx);

  //   //-------- Fetch data --------------
  //   await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  //   await fetchPresaleInfo();
  // });

  // it("create user info", async () => {
  //   const tx = await program.methods
  //     .createUserInfo()
  //     .accounts({
  //       userInfo: user_info[0],
  //       user: userKeypair.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .signers([userKeypair])
  //     .rpc();
  //   console.log("Your transaction signature", tx);

  //   //-------- Fetch data --------------
  //   await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  //   await fetchUserInfo();
  // });

  // it("buy token", async () => {
  //   const amount = new BN(3500);  // 0.0000035 sol -> 1 choobin
  //   const tx = await program.methods
  //     .buyToken(amount)
  //     .accounts({
  //       presaleInfo: presale_info[0],
  //       presaleInfoMintAta: presale_info_mint_ata,
  //       userInfo: user_info[0],
  //       userInfoMintAta: user_info_mint_ata,
  //       user: userKeypair.publicKey,
  //       mint: mint_pubkey,
  //       treasury: treasury_pubkey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
  //     })
  //     .signers([userKeypair])
  //     .rpc();
  //   console.log("Your transaction signature", tx);

  //   //-------- Fetch data --------------
  //   await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  //   await fetchPresaleInfo();
  //   await fetchUserInfo();
  // });

  // it("claim token", async () => {
  //   const tx = await program.methods
  //     .claim()
  //     .accounts({
  //       presaleInfo: presale_info[0],
  //       userInfo: user_info[0],
  //       userInfoMintAta: user_info_mint_ata,
  //       user: userKeypair.publicKey,
  //       mint: mint_pubkey,
  //       userMintAta: user_mint_ata,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
  //     })
  //     .signers([userKeypair])
  //     .rpc();
  //   console.log("Your transaction signature", tx);

  //   //-------- Fetch data --------------
  //   await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  //   await fetchPresaleInfo();
  //   await fetchUserInfo();
  // });

});
