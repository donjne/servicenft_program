import * as anchor from "@coral-xyz/anchor";
import * as web3 from "@solana/web3.js";
import type { Marketplace } from "../target/types/marketplace";
describe("service_market", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Marketplace as anchor.Program<Marketplace>;
  
  // Initialize a prediction market
  it("Initializes service market", async () => {
    const SERVICE_MARKET_SEED = "servicemarket";
    const [serviceMarket] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from(SERVICE_MARKET_SEED)],
      program.programId
    );

  const context = {
        user: program.provider.publicKey,
        marketplace: serviceMarket,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
  }
    const tx = await program.methods.initializeMarketplace().accounts(context).signers([program.provider.wallet.payer]).rpc()

    console.log("Prediction market initialized. Transaction:", tx);
  });

});

https://v2.akord.com/public/vaults/active/xdmaYXBiPF6GS9g0yf3vg7JumVNwWQ_c0aa0Mdz0SnE/gallery#3a540011-9043-4980-b458-8e7d16904d79