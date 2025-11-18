import * as anchor from "@coral-xyz/anchor";

(async () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.GamaedtechProgram;

  const [statsPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stats")],
    program.programId,
  );

  await program.methods.initStats()
    .accounts({
      stats: statsPda,
      authority: program.provider.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("Stats initialized:", statsPda.toString());
})();
