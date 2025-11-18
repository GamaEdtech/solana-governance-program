import * as anchor from "@coral-xyz/anchor";

(async () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.GamaedtechProgram;

  const [statsPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stats")],
    program.programId
  );

  const connection = program.provider.connection;

  // 1 Get size before realloc
  const before = await connection.getAccountInfo(statsPda);
  console.log("Before realloc size:", before?.data.length);

  // 2 Run realloc instruction
  try {
    await program.methods.closeStats().accounts({
      stats: statsPda,
      authority: program.provider.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();
    console.log("Close instruction executed.");
  } catch (err) {
    console.log("Close failed:", err);
    return;
  }

  // 3 Get size after realloc
  const after = await connection.getAccountInfo(statsPda);
  console.log("After realloc size:", after?.data.length);

  console.log("Stats PDA:", statsPda.toString());
})();
