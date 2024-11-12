import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OtcSwaps } from "../target/types/otc_swaps";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  createAssociatedTokenAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("otc-swaps", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.OtcSwaps as Program<OtcSwaps>;

  let seller = Keypair.generate();
  let buyer = Keypair.generate();
  let tokenMint: PublicKey;
  let sellerTokenAccount: PublicKey;
  let buyerRecipientAccount: PublicKey;
  let swapEscrowAccount: PublicKey;
  let swapAccount: PublicKey;
  const amount = 1000000;
  const expiryTimestamp = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now
  const whitelistedBuyers = [buyer.publicKey];

  async function airdropWithRetry(connection, publicKey, lamports, retries = 5) {
    for (let i = 0; i < retries; i++) {
      try {
        const signature = await connection.requestAirdrop(publicKey, lamports);
        await connection.confirmTransaction(signature, "finalized");
        return;
      } catch (error) {
        console.warn(`Airdrop failed, retrying... (${i + 1}/${retries})`);
        await new Promise((resolve) => setTimeout(resolve, 1000 * Math.pow(2, i)));
      }
    }
    throw new Error("Airdrop failed after multiple retries");
  }

  before(async () => {
    const provider = anchor.AnchorProvider.env();
  
    // Retry airdrop for seller
    await airdropWithRetry(provider.connection, seller.publicKey, 2 * 1e9);
  
    // Retry airdrop for buyer
    await airdropWithRetry(provider.connection, buyer.publicKey, 2 * 1e9);
  
    // Create token mint
    tokenMint = await createMint(
      provider.connection,
      seller,
      seller.publicKey,
      null,
      9
    );
  
    // Create associated token accounts for seller and buyer
    sellerTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      seller,
      tokenMint,
      seller.publicKey
    );
    buyerRecipientAccount = await createAssociatedTokenAccount(
      provider.connection,
      buyer,
      tokenMint,
      buyer.publicKey
    );
  
    // Create the escrow account as an associated token account under the swap account authority
    swapEscrowAccount = await createAssociatedTokenAccount(
      provider.connection,
      seller,
      tokenMint,
      seller.publicKey // Replace this with the PDA if required
    );
  
    // Mint tokens to the seller's account
    await mintTo(
      provider.connection,
      seller,
      tokenMint,
      sellerTokenAccount,
      seller,
      amount
    );
  });

  it("Initializes a swap", async () => {
    // Derive swap account address
    [swapAccount] = await PublicKey.findProgramAddressSync(
      [Buffer.from("swap"), seller.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .initializeSwap(
        new anchor.BN(amount),
        new anchor.BN(expiryTimestamp),
        whitelistedBuyers,
        buyerRecipientAccount
      )
      .accounts({
        seller: seller.publicKey,
        swap: swapAccount,
        sellerTokenAccount: sellerTokenAccount,
        tokenMint: tokenMint,
        swapTokenAccount: swapEscrowAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([seller])
      .rpc();

    console.log("Swap initialized with tx:", tx);

    // Fetch swap account and assert initialized state
    const swapState = await program.account.swapAccount.fetch(swapAccount);
    assert.equal(swapState.amount.toNumber(), amount);
    assert.equal(swapState.isActive, true);
    assert.equal(swapState.expiryTimestamp.toNumber(), expiryTimestamp);
  });

  it("Executes the swap", async () => {
    const tx = await program.methods
      .executeSwap(buyerRecipientAccount)
      .accounts({
        buyer: buyer.publicKey,
        swap: swapAccount,
        swapTokenAccount: swapEscrowAccount,
        buyerRecipientAccount: buyerRecipientAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([buyer])
      .rpc();

    console.log("Swap executed with tx:", tx);

    // Fetch swap account and assert completed state
    const swapState = await program.account.swapAccount.fetch(swapAccount);
    assert.equal(swapState.isActive, false);

    // Confirm tokens were transferred to the buyer's recipient account
    const buyerBalance = await program.provider.connection.getTokenAccountBalance(
      buyerRecipientAccount
    );
    assert.equal(buyerBalance.value.amount, amount.toString());
  });

  it("Cancels the swap", async () => {
    // Reset swap for cancellation test
    await program.methods
      .initializeSwap(
        new anchor.BN(amount),
        new anchor.BN(expiryTimestamp),
        whitelistedBuyers,
        buyerRecipientAccount
      )
      .accounts({
        seller: seller.publicKey,
        swap: swapAccount,
        sellerTokenAccount: sellerTokenAccount,
        tokenMint: tokenMint,
        swapTokenAccount: swapEscrowAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([seller])
      .rpc();

    const tx = await program.methods
      .cancelSwap()
      .accounts({
        seller: seller.publicKey,
        swap: swapAccount,
        swapTokenAccount: swapEscrowAccount,
        sellerRecipientAccount: sellerTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([seller])
      .rpc();

    console.log("Swap canceled with tx:", tx);

    // Fetch swap account and assert canceled state
    const swapState = await program.account.swapAccount.fetch(swapAccount);
    assert.equal(swapState.isActive, false);

    // Confirm tokens were returned to the seller's token account
    const sellerBalance = await program.provider.connection.getTokenAccountBalance(
      sellerTokenAccount
    );
    assert.equal(sellerBalance.value.amount, amount.toString());
  });
});
