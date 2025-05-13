import * as dotenv from "dotenv";
dotenv.config();

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { Irma } from "../target/types/irma";

describe("irma", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.irma as Program<Irma>;

  let statePda: PublicKey;

  before(async () => {
    // Derive the PDA for the state account
    [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );
    console.log("State PDA:", statePda.toString());
  });

  it("Initializes the state account", async () => {
    // Call the initialize method to create and initialize the state account
    const tx = await program.methods
      .initialize()
      .accounts({
        state: statePda, // The PDA derived state account
      })
      .rpc();

    console.log("Initialization TX:", tx);

    // Optionally confirm the account was created
    try {
      const stateAccount = await program.account.state.fetch(statePda);
      console.log("Fetched state account:", stateAccount);
    } catch (error) {
      console.error("Error fetching state account:", error);
    }
  });

  // it("Calls hello", async () => {
  //   const tx = await program.methods
  //     .hello()
  //     .accounts({
  //       state: statePda, // Pass the initialized state account
  //     })
  //     .rpc();

  //   console.log("Hello TX:", tx);
  // });
});
