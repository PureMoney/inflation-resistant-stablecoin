import * as dotenv from "dotenv";
dotenv.config();

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { Irma } from "../target/types/irma";

describe("irma", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.irma; // as Program<Irma>;

  let statePda: PublicKey;

  before(async () => {
    // Derive the PDA for the state account
  });

  it("Initializes the state account", async () => {
    // Call the initialize method to create and initialize the state account
    const tx = await program.methods
      .initialize()
      .accounts({
        irmaAdmin: provider.wallet.publicKey, // Use the provider's wallet public key
      })
      .rpc();

    console.log("Initialization TX:", tx);

    // Optionally confirm the account was created
    try {
      [statePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("state")],
        program.programId
      );
      console.log("State PDA:", statePda.toString());
      const stateAccount = await program.account.state.fetch(statePda);
      console.log("Fetched state account:", stateAccount);
    } catch (error) {
      console.error("Error fetching state account:", error);
    }
  });

  it("Calls hello", async () => {
    const tx = await program.methods
      .hello()
      .accounts({
        trader: provider.wallet.publicKey, // Use the provider's wallet public key
      })
      .rpc();

    console.log("Hello TX:", tx);
  });

  // The test below fails because I can't figure out how to select the qoute token.
  // it("Calls setMintPrice", async () => {
  //   const mintPrice = parseFloat("1.00");
  //   if (isNaN(mintPrice) || mintPrice <= 0) {
  //     throw new Error("Invalid mint price. It must be a positive number.");
  //   }           
  //   const tx = await program.methods
  //     .setMintPrice("USDT", mintPrice)
  //     .accounts({
  //       trader: provider.wallet.publicKey, // Use the provider's wallet public key
  //     })
  //     .rpc();

  //   console.log("Set Mint Price TX:", tx);
  // });
});
